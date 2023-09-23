use std::collections::{HashMap, HashSet, VecDeque, BTreeMap};
use std::io::Write;
use std::{fs::File, io::BufReader};

use regex::Regex;
use serde_derive::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::burrito::path_cache::PathCache;
use crate::burrito::types::{SystemId, StargateId, ConstellationId, StarId};
use crate::burrito::utils;

#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SystemContext {
    /// Version of this SystemContext was created in
    ///
    /// A version bump signals map changes and signals that cached paths should be
    /// cleared.
    version: u64,
    /// Current system for which to provide alerts
    #[serde(default)]
    current_systems: HashSet<String>,
    /// Current system IDs
    #[serde(skip)]
    current_system_ids: HashSet<SystemId>,
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
    pub fn new(sys_name: Option<String>, sys_map: &SystemMap) -> Self {
        let mut ctx = load_saved_context(sys_name);
        for system_name in &ctx.current_systems {
            if let Some(id) = get_system_id(system_name, sys_map) {
                ctx.current_system_ids.insert(id);
            }
            else {
                eprintln!("Unrecognized system: {}", &system_name);
            }
        }
        ctx
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

    fn distances(&mut self, other_system: String, sys_map: &SystemMap) -> BTreeMap<Distance, SystemId> {
        let mut results: BTreeMap<Distance, SystemId> = BTreeMap::new();
        for my_sys_id in &self.current_system_ids {
            let my_sys_id = my_sys_id.to_owned();
            let other_sys_id = get_system_id(&other_system, sys_map);
            if other_sys_id == None {
                continue;
            }
            let other_sys_id = other_sys_id.unwrap();
            let key = (my_sys_id, other_sys_id);
            if let Some(path) = self.path_cache.search(&key) {
                self.save();
                results.insert(path, my_sys_id);
            }
            else {
                let computed_distance = match compute_distance(my_sys_id, other_sys_id, sys_map) {
                    Some(path) => Distance::Route { route: (path.route.len() - 1) as u32 },
                    None => Distance::NoRoute,
                };
                self.path_cache.insert(key, computed_distance.clone());
                self.save();
                results.insert(computed_distance, my_sys_id);
            }
        }
        results
    }

    pub fn process_message(&mut self, message: String, sys_map: &SystemMap) -> BTreeMap<Distance, SystemId> {
        for word in message.split(" ") {
            let word = word.to_owned().replace("*", "");
            if word.len() <= 2 {
                continue;
            }
            if let Some(_) = get_system_id(&word, sys_map) {
                return self.distances(
                    word.to_owned(),
                    sys_map
                );
            }
        }
        BTreeMap::new()
    }

    pub fn get_current_systems(&self) -> &HashSet<String> {
        &self.current_systems
    }

    pub fn get_current_system_ids(&self) -> &HashSet<SystemId> {
        &self.current_system_ids
    }

}

fn get_system_id(sys_name: &str, sys_map: &SystemMap) -> Option<SystemId> {
    if let Some(entry) =
        sys_map.systems.iter()
        .find(|sys| sys.1.name == sys_name) {
        return Some(entry.0.to_owned());
    }
    None
}

#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SystemMap {
    #[serde_as(as = "HashMap<serde_with::json::JsonString, _>")]
    systems: HashMap<SystemId, System>,
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
    if let Some(sys) = current_system {
        ctx.current_systems.insert(sys);
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

const J_SPACE_REGEX: &str = r#"^(J[0-9]{6}|Thera|Polaris|A821-A|J7HZ-F|UUA-F4)$"#;
//const POCHVEN_REGION_ID: u32 = 10000070;// TODO: Handle Pochven base cases

fn compute_distance(start_id: SystemId, end_id: SystemId, sys_map: &SystemMap) -> Option<Route> {
    let start_system = sys_map.systems.get(&start_id).unwrap();
    let end_system = sys_map.systems.get(&end_id).unwrap();

    let start_str = start_system.name.as_str();
    let end_str = end_system.name.as_str();
    // Handle base cases of unreachable systems
    let regex = Regex::new(J_SPACE_REGEX).unwrap();
    if regex.captures(start_str).is_some() {
        return None;
    }
    if regex.captures(end_str).is_some() {
        return None;
    }

    let mut visited: HashSet<SystemId> = HashSet::new();
    let mut queue: VecDeque<SystemId> = VecDeque::new();
    let mut parent_map: HashMap<SystemId, SystemId> = HashMap::new();

    queue.push_back(start_id);
    visited.insert(start_id);

    while let Some(curr_id) = queue.pop_front() {

        if curr_id == end_id {
            let mut path = vec![curr_id];
            let mut next_id = curr_id;

            while let Some(&parent_id) = parent_map.get(&next_id) {
                path.push(parent_id);
                next_id = parent_id;
            }

            path.reverse();
            return Some(Route { route: path });

        }

        if let Some(curr_sys) = sys_map.systems.get(&curr_id) {
            curr_sys.stargates.iter().map(|sg| sg.destination.system_id).for_each(|neighbor_id| {
                if !visited.contains(&neighbor_id) {
                    visited.insert(neighbor_id);
                    queue.push_back(neighbor_id);
                    parent_map.insert(neighbor_id, curr_id);
                }
            });
        }
    }
    None
}

#[derive(Clone, Debug, Eq, Hash, Default, Deserialize, PartialEq, Serialize)]
struct Route {
    pub route: Vec<SystemId>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct System {
    #[serde(default)]
    pub constellation_id: ConstellationId,
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
    pub star_id: StarId,
    #[serde(default)]
    pub stargates: Vec<Stargate>,
    #[serde(default)]
    pub stations: Vec<u64>,
    #[serde(default)]
    pub system_id: SystemId,
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
    pub stargate_id: StargateId,
    #[serde(default)]
    pub system_id: SystemId,
    #[serde(default)]
    pub type_id: u64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StargateDestination {
    #[serde(default)]
    pub stargate_id: StargateId,
    #[serde(default)]
    pub system_id: SystemId,
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
    pub fn get_route(&self) -> u32 {
        match self {
            Distance::NoRoute => u32::MAX,
            Distance::RouteFetchErr => u32::MAX - 1,
            Self::Route { route } => route.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::{HashSet, HashMap}, io::BufReader, fs::File, time::Instant};
    use regex::Regex;
    use rand::Rng;
    use crate::burrito::{systems::compute_distance, types::SystemId};

    use super::{SystemMap, Route, J_SPACE_REGEX, System};

    #[test]
    fn test_pathfinding() {
        let mut rng = rand::thread_rng();
        let sys_map: SystemMap = load_system_map();
        let systems: Vec<System> =
            sys_map.systems.iter().map(|e| e.1.to_owned()).collect();

        // Test same system route (Jita)
        assert_eq!(vec![SystemId(30000142)], compute_distance(30000142.into(), 30000142.into(), &sys_map).unwrap().route);

        // Test unreachable (Jita -> Thera)
        assert!(compute_distance(30000142.into(), 31000005.into(), &sys_map).is_none());
        assert!(compute_distance(31000005.into(), 30000142.into(), &sys_map).is_none());

        // Thera -> Polaris
        assert!(compute_distance(31000005.into(), 30000380.into(), &sys_map).is_none());

        // Test known route (Jita -> Amarr)
        assert_eq!(12, compute_distance(30000142.into(), 30002187.into(), &sys_map).unwrap().route.len());
        // 1DQ1-A -> Sakht
        assert_eq!(12, compute_distance(30004759.into(), 30004299.into(), &sys_map).unwrap().route.len());

        // Test randomly selected systems
        let mut random_test_systems: HashMap<SystemId, SystemId> = HashMap::new();
        for _ in 0..2000 {
            let i1 = rng.gen_range(0..systems.len());
            let i2 = rng.gen_range(0..systems.len());
            random_test_systems.insert(systems[i1].system_id, systems[i2].system_id);
        }
        let time_before = Instant::now();
        for case in random_test_systems {
            let expected = find_route_unoptimized(case.0, case.1, &sys_map);
            let actual = compute_distance(case.0, case.1, &sys_map);
            assert_eq!(expected, actual);
        }
        let time = (Instant::now() - time_before).as_secs_f64();
        eprintln!("Route computations finished in {} seconds", time);
    }

    fn find_route_unoptimized(start_id: SystemId, end_id: SystemId, sys_map: &SystemMap) -> Option<Route> {
        let start_str = sys_map.systems.get(&start_id)
            .unwrap().name.as_str();
        let end_str = sys_map.systems.get(&end_id)
            .unwrap().name.as_str();
        let regex = Regex::new(J_SPACE_REGEX).unwrap();
        if regex.captures(start_str).is_some() {
            return None;
        }
        if regex.captures(end_str).is_some() {
            return None;
        }
        let mut visited: HashSet<SystemId> = HashSet::new();
        let mut queue: Vec<Vec<SystemId>> = vec![vec![start_id]];
        while queue.len() > 0 {
            let route = queue.remove(0);
            let curr_id = *route.last().unwrap();
            if curr_id == end_id {
                return Some(Route { route });
            }
            else if visited.contains(&curr_id) {
                continue;
            }
            visited.insert(curr_id);
            if let Some(curr_sys) = sys_map.systems.get(&curr_id) {
                curr_sys.stargates.iter().map(|sg| sg.destination.system_id)
                    .for_each(|neighbor_id| {
                        let mut candidate_route = route.clone();
                        candidate_route.push(neighbor_id);
                        queue.push(candidate_route);
                    });
            }
        }
        return None;
    }

    fn load_system_map() -> SystemMap {
        let file = File::open("data/systems.json").unwrap();
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap()
    }

}
