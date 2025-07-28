use std::{collections::HashSet, path::Path};

use clap::Parser;
use conversion::sumo::{XmlReader, edges_reader::SumoEdgesReader, trips_reader::SumoTripsReader};

fn main() {
    let args = Args::parse();

    let edges_path = Path::new(&args.edges);
    let trips_path = Path::new(&args.trips);

    println!("Reading trips from: {}", trips_path.display());
    let trips = SumoTripsReader::read(trips_path).unwrap();
    println!("Found {} trips", trips.vehicles.len());

    println!("Reading edges from: {}", edges_path.display());
    let edges = SumoEdgesReader::read(edges_path).unwrap();

    println!("Preprocess edges...");
    let edge_set: HashSet<String> = edges.edges.iter().map(|e| e.id.clone()).collect();
    println!("Found {} edges", edge_set.len());

    let mut missing_edges = 0;

    println!("Process trips...");
    for trip in &trips.vehicles {
        if !edge_set.contains(&trip.from) {
            if missing_edges < 10 {
                println!("Trip {} has a \"from\" edge that does not exist: from {}", trip.id, trip.from);
            }
            missing_edges += 1;
        }

        if !edge_set.contains(&trip.to) {
            if missing_edges < 10 {
                println!("Trip {} has a \"to\" edge that does not exist: to {}", trip.id, trip.to);
            }
            missing_edges += 1;
        }
    }

    if missing_edges > 0 {
        println!("Found {} missing edges in trips", missing_edges);
    } else {
        println!("All trip edges are present");
    }
}

/// Command-line arguments for counting connections and whether they are complete or not
#[derive(Parser, Debug)]
#[command(version, about = "Sumo Connection Counter options", long_about = None)]
pub struct Args {
    #[arg(long = "edges")]
    pub edges: String,

    #[arg(long = "trips")]
    pub trips: String,
}
