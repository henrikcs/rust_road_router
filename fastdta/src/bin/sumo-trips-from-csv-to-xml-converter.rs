use std::{fs::remove_dir_all, path::Path};

use clap::Parser;
use conversion::sumo::{
    EDG_XML, FileReader, FileWriter, TRIPS_XML,
    edges_reader::SumoEdgesReader,
    sumo_to_td_graph_converter::convert_sumo_to_routing_kit_and_queries,
    trips::{Trip, TripsDocumentRoot},
    trips_reader::MatsimCsvTripsReader,
    trips_writer::SumoTripsWriter,
};
use fastdta::{
    customize::customize,
    preprocess::{get_cch, preprocess, run_inertial_flow_cutter},
    query::get_paths_with_cch,
};

use rust_road_router::{algo::catchup::Server, datastr::graph::floating_time_dependent::TDGraph, io::Reconstruct};
/// Given an xml file containing sumo edges <edges>, converts a matsim csv file <trips> to a SUMO trip file with the name <output>
/// <trips> should contain the following headers:
/// tripId, legId, tripBeginTime, locationFrom, locationTo
/// the SUMO trip file is an xml file with the following structure:
/// <routes>
/// <trip id="<tripId>-<legId>" depart="<convert_to_seconds_since_midnight(<tripBeginTime>)>" from="<parse_location(<locationFrom>)>" to="<parse_location(<locationTo>)>" departLane="best" departSpeed="max" departPos="base"/>
/// ...
/// </routes>
///
fn main() {
    let args = Args::parse();

    let input_dir = Path::new(&args.input);
    let input_prefix = &args.input_prefix;
    let edges_path = input_dir.join(format!("{}{}", &input_prefix, EDG_XML));
    let trips_path = Path::new(&args.trips);
    let output_path = Path::new(&args.output);
    let output_trips_file = output_path.join(format!("{}{}", &input_prefix, TRIPS_XML));

    println!("Reading edges from: {}", edges_path.display());
    let edges = SumoEdgesReader::read(&edges_path).expect("Failed to read edges");

    println!("Reading trips from: {}", trips_path.display());
    // read trips from csv file
    let mut trips = MatsimCsvTripsReader::read(&trips_path).expect("Failed to read trips");
    // sort trips by departure time

    println!("Sorting trips by departure time...");
    trips.sort_by_key(|trip| trip.trip_begin_time.clone());
    let input_trips_count = trips.len();

    // create a hashset of edge IDs for quick lookup
    let edge_ids: std::collections::HashSet<&String> = edges.edges.iter().map(|edge| &edge.id).collect();

    println!("Filtering trips to only include those with valid edges...");
    // filter trips to only include those with valid edges
    let unchecked_sumo_trips: Vec<Trip> = trips
        .iter()
        .map(|trip| trip.to_sumo_trip())
        .filter(|trip| edge_ids.contains(&trip.from) && edge_ids.contains(&trip.to))
        .collect();

    let unchecked_sumo_trips_document_root = TripsDocumentRoot {
        trips: unchecked_sumo_trips.clone(),
    };

    println!("Preprocessing graph for filtering trips which can be routed on the graph...");

    let temp_dir_name = "tmp";
    let temp_cch_dir = output_path.join(temp_dir_name);
    let temp_trips_file = temp_cch_dir.join(format!("temp_{}{}", input_prefix, conversion::sumo::TRIPS_XML));

    std::fs::create_dir_all(&temp_cch_dir).expect(format!("Failed to create temporary CCH directory {}", temp_cch_dir.display()).as_str());

    println!("Writing temporary SUMO trips file...");
    // output the results as a trips file
    SumoTripsWriter::write(&temp_trips_file, &unchecked_sumo_trips_document_root).expect("Failed to write trips");

    convert_sumo_to_routing_kit_and_queries(&input_dir, &input_prefix, &temp_trips_file, &temp_cch_dir).unwrap();

    // create a subprocess which runs the bash script: "flow_cutter_cch_cut_order.sh <output_dir>" to create node rankings for the TD-CCH
    run_inertial_flow_cutter(&temp_cch_dir, 42, std::thread::available_parallelism().unwrap().get() as i32).unwrap();

    // run catchup preprocessing
    preprocess(&temp_cch_dir).unwrap();

    let graph = TDGraph::reconstruct_from(&temp_cch_dir).unwrap();
    let cch = get_cch(&temp_cch_dir, &graph);
    let customized_graph = customize(&cch, &graph);

    println!("Calculating paths...");
    let (shortest_paths, _, _) = get_paths_with_cch(&cch, &customized_graph, &temp_cch_dir, &graph);

    // filter trips which can be routed on the graph
    println!("Filter trips according to paths...");
    // filter out trips which do not have a path (shortest path is empty)
    let filtered_trips: Vec<Trip> = unchecked_sumo_trips
        .into_iter()
        .zip(shortest_paths.into_iter())
        .filter(|(_, path)| !path.is_empty())
        .map(|(trip, _)| trip)
        .collect();

    // remove the temporary CCH directory
    remove_dir_all(&temp_cch_dir).unwrap();

    let filtered_count = filtered_trips.len();

    println!("Filtered {} trips to {} valid trips", input_trips_count, filtered_count);

    // create a TripsDocumentRoot from the filtered trips
    let trips = conversion::sumo::trips::TripsDocumentRoot { trips: filtered_trips };

    // output the results as a trips file
    SumoTripsWriter::write(&output_trips_file, &trips).expect("Failed to write trips");
    println!("Wrote filtered trips to: {}", output_trips_file.display());
}

/// Command-line arguments for counting connections and whether they are complete or not
#[derive(Parser, Debug)]
#[command(version, about = "Sumo Connection Counter options", long_about = None)]
pub struct Args {
    /// directory which contains .nod.xml, .trips.xml and .trips.csv files
    #[arg(long = "input")]
    pub input: String,

    /// Name of the *.<type>.xml files of the sumo inputs, such as <prefix>.nod.xml and <prefix>.edg.xml inside the inputs directory
    #[arg(long = "input-prefix")]
    pub input_prefix: String,

    /// path to the matsim csv trips file
    #[arg(long = "trips")]
    pub trips: String,

    // path where <prefix>.trips.xml will be written
    #[arg(long = "output")]
    pub output: String,
}
