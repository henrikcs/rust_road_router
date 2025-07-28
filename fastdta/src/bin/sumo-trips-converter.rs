use std::{collections::HashSet, path::Path};

use clap::Parser;
use conversion::sumo::{XmlReader, edges_reader::SumoEdgesReader, trips_reader::SumoTripsReader};

fn main() {
    let args = Args::parse();

    let edges_path = Path::new(&args.edges);
    let nodes_path = Path::new(&args.nodes);
    let trips_path = Path::new(&args.trips);
}

/// Command-line arguments for counting connections and whether they are complete or not
#[derive(Parser, Debug)]
#[command(version, about = "Sumo Connection Counter options", long_about = None)]
pub struct Args {
    #[arg(long = "edges")]
    pub edges: String,

    #[arg(long = "nodes")]
    pub nodes: String,

    #[arg(long = "trips")]
    pub trips: String,
}
