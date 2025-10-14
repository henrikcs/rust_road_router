use std::{
    fs,
    path::{Path, PathBuf},
};

/// trips_file has the following format: "<name>[.trips].xml"
/// the routes file in iteration <iteration> should have the following name:
/// "<name>_<iteration>.rou.xml"
pub fn get_routes_file_name_in_iteration(trips_file: &Path, iteration: u32) -> String {
    let mut file_stem = trips_file.file_stem().unwrap().to_str().unwrap();
    file_stem = if file_stem.ends_with(".trips") {
        &file_stem[..file_stem.len() - 6]
    } else {
        file_stem
    };
    let routes_file_name = format!("{}_{}.rou.xml", file_stem, format!("{:0>3}", iteration));
    routes_file_name
}

pub fn get_meandata_file(dir: &Path) -> PathBuf {
    fs::read_dir(dir)
        .unwrap()
        .find(|entry| {
            // check if entry is a file
            entry.is_ok()
                && entry.as_ref().unwrap().file_type().unwrap().is_file()
                && entry
                    .as_ref()
                    .unwrap()
                    .file_name()
                    .to_str()
                    .unwrap()
                    // check if file name starts with "dump_" and ends with ".xml"
                    .starts_with("dump_")
                && entry.as_ref().unwrap().file_name().to_str().unwrap().ends_with(".xml")
        })
        .map(|entry| entry.unwrap().path())
        .unwrap()
}
