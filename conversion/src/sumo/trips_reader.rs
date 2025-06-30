use std::{error::Error, fs};

use crate::sumo::trips::{Trip, Trips};

pub trait TripsReader {
    fn read(output_file: &str) -> Result<Vec<Trip>, Box<dyn Error>>;
}

pub struct SumoTripsReader {}

impl TripsReader for SumoTripsReader {
    fn read(file: &str) -> Result<Vec<Trip>, Box<dyn Error>> {
        let f = fs::read_to_string(file)?;
        let trips: Trips = serde_xml_rs::from_str(&f).unwrap();

        Ok(trips.trips)
    }
}
