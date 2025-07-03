use std::{error::Error, fs};

use crate::sumo::{nodes::NodesDocumentRoot, XmlReader};

pub struct SumoNodesReader {}

impl XmlReader for SumoNodesReader {
    type R = NodesDocumentRoot;

    fn read(file: &str) -> Result<NodesDocumentRoot, Box<dyn Error>> {
        let f = fs::read_to_string(file)?;
        let n: NodesDocumentRoot = serde_xml_rs::from_str(&f).unwrap();

        Ok(n)
    }
}
