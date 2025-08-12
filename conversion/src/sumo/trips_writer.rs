use std::{error::Error, fs::File, io::Write, path::Path};

use crate::sumo::{trips::TripsDocumentRoot, FileWriter};

pub struct SumoTripsWriter {}

impl FileWriter for SumoTripsWriter {
    type R = TripsDocumentRoot;

    fn write(output_file: &Path, trips: &TripsDocumentRoot) -> Result<(), Box<dyn Error>> {
        let file = File::create(output_file);

        let res = serde_xml_rs::to_string(trips).unwrap();

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
