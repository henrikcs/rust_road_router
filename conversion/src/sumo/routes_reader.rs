use std::{error::Error, fs, path::Path};

use crate::sumo::{routes::RoutesDocumentRoot, XmlReader};

pub struct SumoRoutesReader {}

impl XmlReader for SumoRoutesReader {
    type R = RoutesDocumentRoot;

    fn read(file: &Path) -> Result<RoutesDocumentRoot, Box<dyn Error>> {
        let f = fs::read_to_string(file)?;
        let n: RoutesDocumentRoot = serde_xml_rs::from_str(&f).unwrap();

        Ok(n)
    }
}
