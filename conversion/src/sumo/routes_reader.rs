use std::{error::Error, fs};

use crate::sumo::routes::Routes;

pub trait RoutesReader {
    fn read(output_file: &str) -> Result<Routes, Box<dyn Error>>;
}

pub struct SumoRoutesReader {}

impl RoutesReader for SumoRoutesReader {
    fn read(file: &str) -> Result<Routes, Box<dyn Error>> {
        let f = fs::read_to_string(file)?;
        let trips: Routes = serde_xml_rs::from_str(&f).unwrap();

        Ok(trips)
    }
}
