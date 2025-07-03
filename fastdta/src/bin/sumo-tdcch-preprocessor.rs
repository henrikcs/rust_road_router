use std::path::Path;

use conversion::sumo::sumo_to_td_graph_converter::{convert_sumo_to_td_graph, read_nodes_edges_and_trips_from_plain_xml};

use fastdta::cli;
use fastdta::cli::Parser;
use rust_road_router::datastr::graph::Graph;

fn main() {
    let args = cli::Args::parse();

    let Some(input_dir) = args.input_dir else {
        panic!("No input directory provided to read files from. Use --input-dir <path> to specify a directory containing all of the input files.");
    };

    let Some(input_prefix) = args.input_prefix else {
        panic!("No input prefix provided. Use --input-prefix <prefix> (or -i <prefix>) to specify the prefix of each input file.");
    };

    let Some(_output_dir) = args.output_dir else {
        panic!("No output directory provided. Use --output-dir <path> to specify an output directory for the TD-CCH.");
    };

    let input_dir = Path::new(&input_dir);

    let (nodes, edges, trips) = read_nodes_edges_and_trips_from_plain_xml(input_dir, &input_prefix);

    let (g, edges_by_id) = convert_sumo_to_td_graph(&nodes, &edges);

    dbg!(nodes.nodes.len());
    dbg!(edges.edges.len());
    dbg!(g.num_nodes());
    dbg!(g.num_arcs());

    let number_of_trips = trips.vehicles.len();
    dbg!(number_of_trips);
}
