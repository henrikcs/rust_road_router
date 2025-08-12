use std::{error::Error, fs, path::Path};

use crate::sumo::{nodes::NodesDocumentRoot, FileReader};

pub struct SumoNodesReader {}

impl FileReader for SumoNodesReader {
    type R = NodesDocumentRoot;

    fn read(file: &Path) -> Result<NodesDocumentRoot, Box<dyn Error>> {
        let f = fs::read_to_string(file)?;
        let n: NodesDocumentRoot = serde_xml_rs::from_str(&f).unwrap();

        Ok(n)
    }
}
