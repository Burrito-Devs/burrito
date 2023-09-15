use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::{fs::File, io::BufReader};

use serde_derive::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::burrito::log_watcher::EventType;

use super::burrito_cfg::BurritoCfg;
use super::path_cache::PathCache;
use super::utils;

#[derive(Clone, Copy, Debug, Eq, Hash, Default, Deserialize, Ord, PartialEq, PartialOrd, Serialize)]
pub struct SystemId(u64);

impl std::fmt::Display for SystemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<SystemId> for u64 {
    fn from(sys_id: SystemId) -> Self {
        sys_id.0
    }
}

impl From<u64> for SystemId {
    fn from(value: u64) -> Self {
        SystemId(value)
    }
}

impl From<&u64> for SystemId {
    fn from(value: &u64) -> Self {
        SystemId(*value)
    }
}

#[derive(Clone, Debug, Eq, Default, Deserialize, PartialEq, Serialize)]
pub struct DistanceResults {
    pub character_results: HashMap<String, Distance>,
    pub system_results: HashMap<String, Distance>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SystemContext {
    /// Version of this SystemContext was created in
    ///
    /// A version bump signals map changes and signals that cached paths should be
    /// cleared.
    #[serde(default)]
    version: u64,
    /// Current systems for which to provide alerts
    #[serde(skip)]
    current_systems: HashSet<String>,
    /// Characters for which to provide alerts
    /// This map is character name -> last known system id
    #[serde(skip)]
    current_characters: HashMap<String, u64>,
    /// Cache for computer paths between two systems
    ///
    /// The first time the path between systems is checked, the result will be
    /// cached. This avoids expensive pathfinding for systems that get reported
    /// often. It is likely that players are sticking to a single region, so the
    /// same systems will repeatedly be reported.
    #[serde(default)]
    path_cache: PathCache,
}

// TODO: rewrite this entire thing. Cache entire system map and BFS to find route
// Can optimize out cases such as Polaris or J[0-9]{1,}
impl SystemContext {
    pub fn new(cfg: &BurritoCfg) -> Self {
        load_saved_context(cfg)
    }

    fn save(&self) {
        let mut path = setup_data_dir();
        const CTX_FILE: &str = "/ctx.json";
        path.push_str(CTX_FILE);
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .expect("Failed to save context file");
        f.write_all(serde_json::to_string(&self)
                .expect("Failed to serialize context").as_bytes())
            .expect("Failed to write context to file");
    }

    fn distances(&mut self, event_system: String, sys_map: &SystemMap) -> Option<DistanceResults> {
        let event_sys_id = get_system_id(event_system, sys_map);
        if event_sys_id == None {
            return None;
        }
        let event_sys_id = event_sys_id.unwrap();
        let mut result = DistanceResults::default();
        // TODO: Is to_owned() sane here?
        for system in &self.current_systems {
            let my_sys_id = get_system_id(system.to_owned(), &sys_map);
            let my_sys_id = my_sys_id
                .expect(format!("Invalid system in config: {}", &system).as_str());
            let key = (my_sys_id, event_sys_id);
            let distance = {
                if let Some(path) = self.path_cache.search(&key) {
                    self.save();
                    path.to_owned()
                }
                else {
                    let fetched_distance = fetch_distance(my_sys_id, event_sys_id);
                    self.path_cache.insert(key, fetched_distance.clone());
                    self.save();
                    fetched_distance
                }
            };
            result.system_results.insert(system.to_owned(), distance);
        }
        for character_entry in &self.current_characters {
            let char_sys_id = character_entry.1;
            let key = (char_sys_id.into(), event_sys_id);
            let distance = {
                if let Some(path) = self.path_cache.search(&key) {
                    self.save();
                    path.to_owned()
                }
                else {
                    let fetched_distance = fetch_distance(char_sys_id.into(), event_sys_id);
                    self.path_cache.insert(key, fetched_distance.clone());
                    self.save();
                    fetched_distance
                }
            };
            result.character_results.insert(character_entry.0.to_owned(), distance);
        }
        Some(result)
    }

    pub fn process_message(&mut self, message: String, sys_map: &SystemMap) -> Option<DistanceResults> {
        for word in message.split(" ") {
            let word = word.to_owned().replace("*", "");
            if word.len() < 3 {
                continue;
            }
            if let Some(_) = get_system_id(word.to_owned(), sys_map) {
                return self.distances(
                    word.to_owned(),
                    sys_map
                );
            }
        }
        None
    }

    pub fn get_current_systems(&self) -> &HashSet<String> {
        &self.current_systems
    }

    pub fn set_current_system(&mut self, current_systems: HashSet<String>) {
        self.current_systems = current_systems;
    }

    pub fn get_current_characters(&self) -> &HashMap<String, u64> {
        &self.current_characters
    }

    pub fn set_current_characters(&mut self, current_characters: HashMap<String, u64>) {
        self.current_characters = current_characters;
    }

}

fn get_system_id(sys_name: String, sys_map: &SystemMap) -> Option<SystemId> {
    if let Some(entry) =
        sys_map.systems.iter()
        .find(|sys| sys.1.name == sys_name) {
        return Some(SystemId(entry.0.to_owned()));
    }
    None
}

#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SystemMap {
    #[serde_as(as = "HashMap<serde_with::json::JsonString, _>")]
    systems: HashMap<u64, System>,
}

fn load_saved_context(cfg: &BurritoCfg) -> SystemContext {
    let mut path = setup_data_dir();
    const CTX_FILE: &str = "/ctx.json";
    let path_cache = Default::default();
    let mut ctx = SystemContext::default();
    path.push_str(CTX_FILE);
    if !std::path::Path::new(&path).exists() {
        eprintln!("No saved context found. Creating default context");
        ctx.path_cache = path_cache;
    }
    else {
        let f = File::open(&path)
            .expect("Failed to open saved context file");
        let reader = BufReader::new(f);
        ctx = serde_json::from_reader(reader)
            .expect("Failed to deserialize saved context");
    }
    cfg.sound_config.audio_alerts.iter().for_each(|alert| {
        match &alert.trigger {
            EventType::RangeOfSystem(_, system_name) => {
                ctx.current_systems.insert(system_name.to_owned());
            },
            EventType::RangeOfCharacter(_, char_name) => {
                ctx.current_characters.insert(char_name.to_owned(), 0);
            },
            _ => {},
        }
    });
    ctx.save();
    ctx
}

pub fn load_saved_system_map() -> SystemMap {
    let mut path = setup_data_dir();
    const SYS_MAP_FILE: &str = "/systems.json";
    path.push_str(SYS_MAP_FILE);
    let f = File::open(&path).expect("systems.json file not found");
    let reader = BufReader::new(f);
    let map: SystemMap = serde_json::from_reader(reader).expect("Failed to read systems.json");
    eprintln!("System map loaded with {} systems", map.systems.len());
    map
}

fn setup_data_dir() -> String {
    let full_path = utils::get_burrito_dir();
    if !std::path::Path::new(&full_path).exists() {
        let path = std::path::Path::new(&full_path);
        eprintln!("Data dir not found. Creating directory {}", path.display());
        std::fs::create_dir_all(path).expect("Could not create data dir");
    }
    full_path
}

fn fetch_distance(origin: SystemId, destination: SystemId) -> Distance {
    let request =
        format!("https://esi.evetech.net/latest/route/{}/{}/?datasource=tranquility&flag=shortest", origin, destination);
    let response =
        reqwest::blocking::get(request);
    if response.is_err() {
        eprintln!("Failed to query route {}, {}", origin, destination);
        return Distance::RouteFetchErr;
    }
    let response = response.unwrap();
    // No route found (ex: J-space)
    if response.status().as_u16() == 404u16 {
        return Distance::NoRoute;
    }
    let resp_json: Result<Vec<u32>, reqwest::Error> = response.json();
    if resp_json.is_err() {
        eprintln!("Invalid response for {} -> {}", origin, destination);
        return Distance::RouteFetchErr;
    }
    Distance::Route { route: resp_json.unwrap().len() as u32 - 1 }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct System {
    #[serde(default)]
    pub constellation_id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub planets: Vec<Planet>,
    #[serde(default)]
    pub position: Position,
    #[serde(default)]
    pub security_class: String,
    #[serde(default)]
    pub security_status: f64,
    #[serde(default)]
    pub star_id: u64,
    #[serde(default)]
    pub stargates: Vec<Stargate>,
    #[serde(default)]
    pub stations: Vec<u64>,
    #[serde(default)]
    pub system_id: u64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stargate {
    #[serde(default)]
    pub destination: StargateDestination,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub position: Position,
    #[serde(default)]
    pub stargate_id: u64,
    #[serde(default)]
    pub system_id: u64,
    #[serde(default)]
    pub type_id: u64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StargateDestination {
    #[serde(default)]
    pub stargate_id: u64,
    #[serde(default)]
    pub system_id: u64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Planet {
    #[serde(default)]
    pub asteroid_belts: Vec<i64>,
    #[serde(default)]
    pub moons: Vec<i64>,
    #[serde(default)]
    pub planet_id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
    #[serde(default)]
    pub z: f64,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Distance {
    NoRoute,
    #[default]
    RouteFetchErr,
    Route{route: u32}
}

impl Distance {
    pub fn get_route(&self) -> u32 {
        match self {
            Distance::NoRoute => u32::MAX,
            Distance::RouteFetchErr => u32::MAX - 1,
            Self::Route { route } => route.to_owned(),
        }
    }
}
