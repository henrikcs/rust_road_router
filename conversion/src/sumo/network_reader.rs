use std::{error::Error, fs};

use crate::sumo::network::Network;

pub trait NetworkReader {
    fn read(output_file: &str) -> Result<Network, Box<dyn Error>>;
}

pub struct SumoNetworkReader {}

impl NetworkReader for SumoNetworkReader {
    fn read(file: &str) -> Result<Network, Box<dyn Error>> {
        let f = fs::read_to_string(file)?;
        let n: Network = serde_xml_rs::from_str(&f).unwrap();

        Ok(n)
    }
}
