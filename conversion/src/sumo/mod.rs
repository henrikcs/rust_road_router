use std::{error::Error, path::Path};

pub mod base_types;
pub mod edges;
pub mod edges_reader;
pub mod meandata;
pub mod meandata_reader;
pub mod nodes;
pub mod nodes_reader;
pub mod paths_to_sumo_routes_converter;
pub mod routes;
pub mod routes_writer;
pub mod sumo_to_td_graph_converter;
pub mod trips;
pub mod trips_reader;

pub const EDG_XML: &str = ".edg.xml";
pub const NOD_XML: &str = ".nod.xml";
pub const CON_XML: &str = ".con.xml";
pub const TRIPS_XML: &str = ".trips.xml";
pub const ROUTES: &str = ".rou.xml";
pub const ALT_ROUTES: &str = ".rou.alt.xml";

pub trait XmlReader {
    type R;

    fn read(file: &Path) -> Result<Self::R, Box<dyn Error>>;
}

pub trait XmlWriter {
    type R;

    fn write(output_file: &Path, doc: &Self::R) -> Result<(), Box<dyn Error>>;
}
