use conversion::sumo::network_reader::{NetworkReader, SumoNetworkReader};
use conversion::sumo::routes_reader::{RoutesReader, SumoRoutesReader};
use fastdta::cli;
use fastdta::cli::Parser;

fn main() {
    let args = cli::Args::parse();

    let Some(network_file) = args.net_file else {
        panic!("No network file provided. Use --net-file <path> (or -n <path>) to specify a network file.");
    };

    let Some(trip_file) = args.route_files else {
        panic!("No route file(s) provided. Use --route-files <path> (or -t <path>) to specify route file(s)");
    };

    // let Some(output_dir) = args.output_dir else {
    //     panic!("No output directory provided. Use --output-dir <dir> to specify the output file.");
    // };

    // let Some(output_file) = args.output_file else {
    //     panic!("No output file provided. Use --output-file <path> (or -o <path>) to specify the output file.");
    // };

    let Ok(network) = SumoNetworkReader::read(network_file.as_str()) else {
        panic!("Network could not be read.");
    };

    let network_edges = network.edge.len();
    println!("Number of edges: {network_edges}");

    let Ok(routes) = SumoRoutesReader::read(trip_file.as_str()) else {
        panic!("Routes could not be read.");
    };

    let number_of_routes = routes.content.len();

    println!("Number of routes: {number_of_routes}");
}
