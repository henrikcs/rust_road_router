use std::{error::Error, fs::File, io::Write, path::Path};

use crate::sumo::{routes::RoutesDocumentRoot, XmlWriter};

pub struct SumoRoutesWriter {}

impl XmlWriter for SumoRoutesWriter {
    type R = RoutesDocumentRoot;

    fn write(output_file: &Path, routes: &RoutesDocumentRoot) -> Result<(), Box<dyn Error>> {
        let file = File::create(output_file);

        let res = serde_xml_rs::to_string(routes).unwrap();

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
