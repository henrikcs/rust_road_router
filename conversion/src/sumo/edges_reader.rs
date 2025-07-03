use std::{error::Error, fs};

use crate::sumo::{edges::EdgesDocumentRoot, XmlReader};

pub struct SumoEdgesReader {}

impl XmlReader for SumoEdgesReader {
    type R = EdgesDocumentRoot;

    fn read(file: &str) -> Result<EdgesDocumentRoot, Box<dyn Error>> {
        let f = fs::read_to_string(file)?;
        let trips: EdgesDocumentRoot = serde_xml_rs::from_str(&f).unwrap();

        dbg!(&f);
        dbg!(&trips);

        Ok(trips)
    }
}
