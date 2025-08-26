use std::{error::Error, fs, path::Path};

use crate::sumo::{tripinfo::TripinfosDocumentRoot, FileReader};

pub struct SumoTripinfoReader {}

impl FileReader for SumoTripinfoReader {
    type R = TripinfosDocumentRoot;

    fn read(file: &Path) -> Result<TripinfosDocumentRoot, Box<dyn Error>> {
        let f = fs::read_to_string(file)?;
        let n: TripinfosDocumentRoot = serde_xml_rs::from_str(&f).unwrap();

        Ok(n)
    }
}
