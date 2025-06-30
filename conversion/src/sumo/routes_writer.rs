use std::{error::Error, fs::File, io::Write};

pub trait RoutesWriter {
    fn write(output_file: &str, vehicles: Vec<String>) -> Result<(), Box<dyn Error>>;
}

pub struct SumoRoutesWriter {}

impl RoutesWriter for SumoRoutesWriter {
    fn write(output_file: &str, _vehicles: Vec<String>) -> Result<(), Box<dyn Error>> {
        let file = File::create(output_file);

        let res = ""; //serde_xml_rs::to_string(vehicles).unwrap();

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
