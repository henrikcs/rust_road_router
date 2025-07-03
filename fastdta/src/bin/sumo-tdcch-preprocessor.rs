use conversion::sumo::XmlReader;
use conversion::sumo::edges_reader::SumoEdgesReader;
use conversion::sumo::nodes_reader::SumoNodesReader;
use conversion::sumo::trips_reader::SumoTripsReader;
use fastdta::cli;
use fastdta::cli::Parser;

const EDG_XML: &str = ".edg.xml";
const NOD_XML: &str = ".nod.xml";
const CON_XML: &str = ".con.xml";
const TRIPS_XML: &str = ".trips.xml";

fn main() {
    let args = cli::Args::parse();

    let Some(input_prefix) = args.input_prefix else {
        panic!("No input prefix provided to read files from. Use --input-prefix <prefix> (or -i <prefix) to specify a inputs file.");
    };

    let Ok(edges) = SumoEdgesReader::read((input_prefix.clone() + EDG_XML).as_str()) else {
        panic!("Edges could not be read.");
    };

    let number_of_edges = edges.edges.len();

    dbg!(number_of_edges);

    let Ok(nodes) = SumoNodesReader::read((input_prefix.clone() + NOD_XML).as_str()) else {
        panic!("Edges could not be read.");
    };

    let number_of_nodes = nodes.nodes.len();

    dbg!(number_of_nodes);

    let Ok(trips) = SumoTripsReader::read((input_prefix.clone() + TRIPS_XML).as_str()) else {
        panic!("Trips could not be read.");
    };

    let number_of_trips = trips.vehicles.len();
    dbg!(number_of_trips);
}
