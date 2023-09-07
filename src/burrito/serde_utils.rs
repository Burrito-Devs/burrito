use std::{fs::File, io::{BufWriter, BufReader}};


pub fn read_or_create_default_data_struct<T: for<'a> serde::Deserialize<'a> + serde::Serialize + Default + Clone>(path: &str, filename: &str) -> T {
    // TODO Prefer returning default to panicking
    let home_dir = std::env::var("HOME").expect("$HOME not set");
    const BURRITO_BASE_DIR: &str = "/.burrito/";
    let mut path_builder = String::new();
    path_builder.push_str(home_dir.as_str());
    path_builder.push_str(BURRITO_BASE_DIR);
    path_builder.push_str(path);
    if !std::path::Path::new(&path_builder).exists() {
        let path = std::path::Path::new(&path_builder);
        eprintln!("Directory not found. Creating directory {}", path.display());
        std::fs::create_dir_all(path).expect("Could not create dir");
    }
    path_builder.push_str(filename);
    let def_value: T = Default::default();
    let mut ret_value = def_value.clone();
    if !std::path::Path::new(&path_builder).exists() {
        eprintln!("Data file not found. Creating default value");
        let def_file = File::create(&path_builder).expect("Could not create default data file");
        let writer = BufWriter::new(def_file);
        serde_json::to_writer_pretty(writer, &def_value).expect("Failed to write default data file");
    }
    else {// TODO: HERE! this isn't working. New values do not get added to config file
        let f = File::open(path_builder).expect("Unable to open data file");
        let f_w = f.try_clone().unwrap();
        let reader = BufReader::new(f);
        let loaded_struct: T = serde_json::from_reader(reader).expect("Invalid struct");
        let writer = BufWriter::new(f_w);
        serde_json::to_writer_pretty(writer, &loaded_struct).expect("Failed to update data file");
        ret_value = loaded_struct;
    }
    return ret_value;
}