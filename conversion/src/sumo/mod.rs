use std::{error::Error, path::Path};

pub mod base_types;
pub mod edges;
pub mod edges_reader;
pub mod meandata;
pub mod meandata_reader;
pub mod nodes;
pub mod nodes_reader;
pub mod routes;
pub mod routes_writer;
pub mod sumo_to_td_graph_converter;
pub mod trips;
pub mod trips_reader;
pub trait XmlReader {
    type R;

    fn read(file: &Path) -> Result<Self::R, Box<dyn Error>>;
}

pub trait XmlWriter {
    type R;

    fn write(output_file: &Path, doc: &Self::R) -> Result<(), Box<dyn Error>>;
}
