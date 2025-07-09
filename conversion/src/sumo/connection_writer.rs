use std::{error::Error, fs::File, io::Write, path::Path};

use crate::sumo::{connections::ConnectionsDocumentRoot, XmlWriter};

pub struct SumoConnectionsWriter {}

impl XmlWriter for SumoConnectionsWriter {
    type R = ConnectionsDocumentRoot;

    fn write(output_file: &Path, connections: &ConnectionsDocumentRoot) -> Result<(), Box<dyn Error>> {
        let file = File::create(output_file);

        let res = serde_xml_rs::to_string(connections).unwrap();

        match file {
            Ok(mut f) => {
                f.write_all(res.as_bytes())?;
            }
            Err(e) => {
                eprintln!("Error creating file: {}", e);
                return Err(Box::new(e));
            }
        };

        Ok(())
    }
}
