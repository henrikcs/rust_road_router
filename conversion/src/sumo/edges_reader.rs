use std::{error::Error, fs, path::Path};

use crate::sumo::{edges::EdgesDocumentRoot, XmlReader};

pub struct SumoEdgesReader {}

impl XmlReader for SumoEdgesReader {
    type R = EdgesDocumentRoot;

    fn read(file: &Path) -> Result<EdgesDocumentRoot, Box<dyn Error>> {
        let f = fs::read_to_string(file)?;
        let trips: EdgesDocumentRoot = serde_xml_rs::from_str(&f).unwrap();

        Ok(trips)
    }
}
