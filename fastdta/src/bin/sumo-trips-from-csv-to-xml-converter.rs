use std::path::Path;

use clap::Parser;
use conversion::sumo::{
    EDG_XML, FileReader, FileWriter,
    edges_reader::SumoEdgesReader,
    sumo_to_td_graph_converter::{get_queries_from_trips, get_routing_kit_td_graph_from_sumo, read_nodes_edges_and_connections_from_plain_xml},
    trips::{Trip, TripsDocumentRoot},
    trips_reader::MatsimCsvTripsReader,
    trips_writer::SumoTripsWriter,
};
use fastdta::query::get_paths_with_dijkstra_queries;

use rust_road_router::{algo::dijkstra::query::floating_td_dijkstra::Server, datastr::graph::floating_time_dependent::TDGraph};
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

    let input_dir = Path::new(&args.input);
    let input_prefix = &args.input_prefix;
    let edges_path = input_dir.join(format!("{}{}", &input_prefix, EDG_XML));
    let trips_path = Path::new(&args.trips);
    let output_path = Path::new(&args.output);

    let edges = SumoEdgesReader::read(&edges_path).expect("Failed to read edges");

    // read trips from csv file
    let mut trips = MatsimCsvTripsReader::read(&trips_path).expect("Failed to read trips");
    // sort trips by departure time

    trips.sort_by_key(|trip| trip.trip_begin_time.clone());

    // create a hashset of edge IDs for quick lookup
    let edge_ids: std::collections::HashSet<&String> = edges.edges.iter().map(|edge| &edge.id).collect();

    // filter trips to only include those with valid edges
    let unchecked_sumo_trips: Vec<Trip> = trips
        .iter()
        .map(|trip| trip.to_sumo_trip())
        .filter(|trip| edge_ids.contains(&trip.from) && edge_ids.contains(&trip.to))
        .collect();

    let trips_count = trips.len();

    // filter trips which can be routed on the graph
    let unchecked_sumo_trips_document_root = TripsDocumentRoot {
        trips: unchecked_sumo_trips.clone(),
    };

    let (nodes, edges, connections) = read_nodes_edges_and_connections_from_plain_xml(input_dir, input_prefix);
    let (graph, _, edge_ids_to_index, _) = get_routing_kit_td_graph_from_sumo(&nodes, &edges, &connections);
    let query_data = get_queries_from_trips(&unchecked_sumo_trips_document_root, &edge_ids_to_index);

    let graph = TDGraph::new(graph.0, graph.1, graph.2, graph.3, graph.4);

    let (shortest_paths, _, _) = get_paths_with_dijkstra_queries(
        &mut Server::new(graph.clone()),
        &query_data.1,
        &query_data.2,
        &query_data.3,
        &query_data.4,
        &query_data.5,
        &graph,
    );

    // filter out trips which do not have a path (shortest path is empty)
    let filtered_trips: Vec<Trip> = unchecked_sumo_trips
        .into_iter()
        .zip(shortest_paths.into_iter())
        .filter(|(_, path)| !path.is_empty())
        .map(|(trip, _)| trip)
        .collect();

    let filtered_count = filtered_trips.len();

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
    /// directory which contains .nod.xml, .trips.xml and .trips.csv files
    #[arg(long = "input")]
    pub input: String,

    /// Name of the *.<type>.xml files of the sumo inputs, such as <prefix>.nod.xml and <prefix>.edg.xml inside the inputs directory
    #[arg(long = "edges")]
    pub input_prefix: String,

    /// path to the matsim csv trips file
    #[arg(long = "trips")]
    pub trips: String,

    #[arg(long = "output")]
    pub output: String,
}
