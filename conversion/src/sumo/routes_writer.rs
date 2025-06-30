use std::{error::Error, fs::File, io::Write};

use crate::sumo::routes::{Route, Routes, Vehicle};

pub trait RoutesWriter {
    fn write(output_file: &str, vehicles: Vec<Vehicle>) -> Result<(), Box<dyn Error>>;
}

pub struct SumoRoutesWriter {}

impl SumoRoutesWriter {
    fn get_example_routes() -> Routes {
        Routes {
            xmlns_xsi: "http://www.w3.org/2001/XMLSchema-instance".to_string(),
            schema_location: "http://sumo.dlr.de/xsd/routes_file.xsd".to_string(),
            vehicles: vec![
                Vehicle {
                    id: "0".to_string(),
                    depart: "0.00".to_string(),
                    depart_lane: "best".to_string(),
                    depart_pos: "random".to_string(),
                    depart_speed: "max".to_string(),
                    route: Route {
                        edges: "G3G2 G2F2 F2E2 E2D2 D2C2 C2B2 B2A2 A2A3 A3A4".to_string(),
                    },
                },
                Vehicle {
                    id: "1".to_string(),
                    depart: "1.00".to_string(),
                    depart_lane: "best".to_string(),
                    depart_pos: "random".to_string(),
                    depart_speed: "max".to_string(),
                    route: Route {
                        edges: "C8C9 C9C8 C8C7 C7C6 C6C5 C5C4 C4C3 C3D3".to_string(),
                    },
                },
                Vehicle {
                    id: "2".to_string(),
                    depart: "2.00".to_string(),
                    depart_lane: "best".to_string(),
                    depart_pos: "random".to_string(),
                    depart_speed: "max".to_string(),
                    route: Route {
                        edges: "H2H3 H3H4 H4H5 H5H6 H6G6 G6G7".to_string(),
                    },
                },
            ],
        }
    }
}
impl RoutesWriter for SumoRoutesWriter {
    fn write(output_file: &str, _vehicles: Vec<Vehicle>) -> Result<(), Box<dyn Error>> {
        let file = File::create(output_file);

        let res = serde_xml_rs::to_string(&SumoRoutesWriter::get_example_routes())?;

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
