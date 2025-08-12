use std::path::Path;

use clap::Parser;
use conversion::sumo::{FileReader, FileWriter, edges_reader::SumoEdgesReader, trips::Trip, trips_reader::MatsimCsvTripsReader, trips_writer::SumoTripsWriter};
/// Given an xml file containing sumo edges <edges>, converts a matsim csv file <trips> to a SUMO trip file with the name <output>
/// <trips> should contain the following headers:
/// tripId, legId, tripBeginTime, locationFrom, locationTo
/// the SUMO trip file is an xml file with the following structure:
/// <routes>
/// <trip id="<tripId>-<legId>" depart="<convert_to_seconds_since_midnight(<tripBeginTime>)>" from="<parse_location(<locationFrom>)>" to="<parse_location(<locationTo>)>" departLane="best" departSpeed="max" departPos="base"/>
/// ...
/// </routes>
fn main() {
    let args = Args::parse();

    let edges_path = Path::new(&args.edges);
    let trips_path = Path::new(&args.trips);
    let output_path = Path::new(&args.output);

    let edges = SumoEdgesReader::read(edges_path).expect("Failed to read edges");

    // read trips from csv file
    let trips = MatsimCsvTripsReader::read(trips_path).expect("Failed to read trips");

    // create a hashset of edge IDs for quick lookup
    let edge_ids: std::collections::HashSet<&String> = edges.edges.iter().map(|edge| &edge.id).collect();

    // filter trips to only include those with valid edges
    let filtered_trips: Vec<Trip> = trips
        .iter()
        .map(|trip| trip.to_sumo_trip())
        .filter(|trip| edge_ids.contains(&trip.from) && edge_ids.contains(&trip.to))
        .collect();

    let filtered_count = filtered_trips.len();
    let trips_count = trips.len();

    println!("Filtered {} trips to {} valid trips", trips_count, filtered_count);

    // create a TripsDocumentRoot from the filtered trips
    let trips = conversion::sumo::trips::TripsDocumentRoot { trips: filtered_trips };

    // output the results as a trips file
    SumoTripsWriter::write(output_path, &trips).expect("Failed to write trips");
}

/// Command-line arguments for counting connections and whether they are complete or not
#[derive(Parser, Debug)]
#[command(version, about = "Sumo Connection Counter options", long_about = None)]
pub struct Args {
    #[arg(long = "edges")]
    pub edges: String,

    /// Path to the trips file given as csv with
    #[arg(long = "trips")]
    pub trips: String,

    #[arg(long = "output")]
    pub output: String,
}
