use std::{error::Error, fs::File, io::Write, path::Path};

use crate::sumo::{meandata::MeandataDocumentRoot, FileWriter};

pub struct SumoMeandataWriter {}

impl FileWriter for SumoMeandataWriter {
    type R = MeandataDocumentRoot;

    fn write(output_file: &Path, meandata: &MeandataDocumentRoot) -> Result<(), Box<dyn Error>> {
        let file = File::create(output_file);

        let res = serde_xml_rs::to_string(meandata).unwrap();

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
