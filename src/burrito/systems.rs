use std::collections::HashMap;
use std::io::Write;
use std::{fs::File, io::BufReader};

use serde_derive::{Deserialize, Serialize};
use serde_with::serde_as;

use super::path_cache::PathCache;
use super::utils;

#[derive(Clone, Copy, Debug, Eq, Hash, Default, Deserialize, Ord, PartialEq, PartialOrd, Serialize)]
pub struct SystemId(u64);

impl std::fmt::Display for SystemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SystemContext {
    /// Version of this SystemContext was created in
    ///
    /// A version bump signals map changes and signals that cached paths should be
    /// cleared.
    version: u64,
    /// Current system for which to provide alerts
    current_system: String,
    /// Cache for computer paths between two systems
    ///
    /// The first time the path between systems is checked, the result will be
    /// cached. This avoids expensive pathfinding for systems that get reported
    /// often. It is likely that players are sticking to a single region, so the
    /// same systems will repeatedly be reported.
    path_cache: PathCache,
}

// TODO: rewrite this entire thing. Cache entire system map and BFS to find route
// Can optimize out cases such as Polaris or J[0-9]{1,}
impl SystemContext {
    pub fn new(current_system: Option<String>) -> Self {
        load_saved_context(current_system)
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

    fn distance(&mut self, my_system: String, other_system: String, sys_map: &SystemMap) -> Distance {
        let my_sys_id = get_system_id(my_system, sys_map);
        let other_sys_id = get_system_id(other_system, sys_map);
        if (my_sys_id == None) || (other_sys_id == None) {
            return Distance::RouteFetchErr;
        }
        let key = (my_sys_id.unwrap(), other_sys_id.unwrap());
        if let Some(path) = self.path_cache.search(&key) {
            self.save();
            path.to_owned()
        }
        else {
            let fetched_distance = fetch_distance(my_sys_id.unwrap(), other_sys_id.unwrap());
            self.path_cache.insert(key, fetched_distance.clone());
            self.save();
            return fetched_distance;
        }
    }

    pub fn process_message(&mut self, message: String, sys_map: &SystemMap) -> u32 {
        for word in message.split(" ") {
            let word = word.to_owned().replace("*", "");
            if word.len() <= 2 {
                continue;
            }
            if let Some(_) = get_system_id(word.to_owned(), sys_map) {
                return self.distance(
                    self.current_system.to_owned(),
                    word.to_owned(),
                    sys_map
                ).get_route();
            }
        }
        u32::MAX
    }

    pub fn get_current_system(&self) -> &str {
        &self.current_system
    }

    pub fn set_current_system(&mut self, current_system: &str) {
        self.current_system = current_system.to_owned();
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

fn load_saved_context(current_system: Option<String>) -> SystemContext {
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
    if current_system.is_some() {
        ctx.current_system = current_system.unwrap();
    }
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

#[derive(Clone, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Distance {
    NoRoute,
    #[default]
    RouteFetchErr,
    Route{route: u32}
}

impl Distance {
    fn get_route(&self) -> u32 {
        match self {
            Distance::NoRoute => u32::MAX,
            Distance::RouteFetchErr => u32::MAX - 1,
            Self::Route { route } => route.to_owned(),
        }
    }
}
