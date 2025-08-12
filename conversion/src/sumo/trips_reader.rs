use std::{error::Error, fs, path::Path};

use crate::sumo::{
    trips::{MatsimCsvTrip, TripsDocumentRoot},
    FileReader,
};

pub struct SumoTripsReader {}

pub struct MatsimCsvTripsReader {}

impl FileReader for SumoTripsReader {
    type R = TripsDocumentRoot;

    fn read(file: &Path) -> Result<TripsDocumentRoot, Box<dyn Error>> {
        let f = fs::read_to_string(file)?;
        let n: TripsDocumentRoot = serde_xml_rs::from_str(&f).unwrap();

        Ok(n)
    }
}

impl FileReader for MatsimCsvTripsReader {
    type R = Vec<MatsimCsvTrip>;

    fn read(file: &Path) -> Result<Vec<MatsimCsvTrip>, Box<dyn Error>> {
        let mut rdr = csv::ReaderBuilder::new().has_headers(true).delimiter(b';').from_path(file)?;

        let mut trips = Vec::new();

        for result in rdr.deserialize() {
            let trip: MatsimCsvTrip = result?;
            trips.push(trip);
        }

        Ok(trips)
    }
}
