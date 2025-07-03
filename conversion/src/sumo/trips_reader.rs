use std::{error::Error, fs, path::Path};

use crate::sumo::{trips::TripsDocumentRoot, XmlReader};

pub struct SumoTripsReader {}

impl XmlReader for SumoTripsReader {
    type R = TripsDocumentRoot;

    fn read(file: &Path) -> Result<TripsDocumentRoot, Box<dyn Error>> {
        let f = fs::read_to_string(file)?;
        let n: TripsDocumentRoot = serde_xml_rs::from_str(&f).unwrap();

        Ok(n)
    }
}
