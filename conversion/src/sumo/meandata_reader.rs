use std::{error::Error, fs, path::Path};

use crate::sumo::{meandata::MeandataDocumentRoot, FileReader};

pub struct SumoMeandataReader {}

impl FileReader for SumoMeandataReader {
    type R = MeandataDocumentRoot;

    fn read(output_file: &Path) -> Result<MeandataDocumentRoot, Box<dyn Error>> {
        let f = fs::read_to_string(output_file)?;
        let meandata: MeandataDocumentRoot = serde_xml_rs::from_str(&f).unwrap();

        Ok(meandata)
    }
}
