use std::{fs::File, io::{BufWriter, BufReader, Write}};

use crate::burrito::utils;

// TODO: Refactor bounds for T into a new trait
pub fn read_or_create_default_data_struct<T: for<'a> serde::Deserialize<'a> + serde::Serialize + Default + Clone>(path: &str, filename: &str) -> T {
    let mut path_builder = utils::get_burrito_dir();
    path_builder.push_str("/");
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
    else {
        let f = File::open(&path_builder).expect("Unable to open data file");
        let reader = BufReader::new(f);
        let loaded_struct: T = serde_json::from_reader(reader).expect("Invalid struct");
        let mut f_w = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path_builder)
            .expect("Failed to save context file");
        f_w.write_all(serde_json::to_string_pretty(&loaded_struct)
                .expect("Failed to serialize context").as_bytes())
            .expect("Failed to write context to file");
        ret_value = loaded_struct;
    }
    return ret_value;
}
