use std::{error::Error, fs, path::Path};

use crate::sumo::{connections::ConnectionsDocumentRoot, FileReader};

pub struct SumoConnectionsReader {}

impl FileReader for SumoConnectionsReader {
    type R = ConnectionsDocumentRoot;

    fn read(output_file: &Path) -> Result<ConnectionsDocumentRoot, Box<dyn Error>> {
        let f = fs::read_to_string(output_file)?;
        let meandata: ConnectionsDocumentRoot = serde_xml_rs::from_str(&f).unwrap();

        Ok(meandata)
    }
}
