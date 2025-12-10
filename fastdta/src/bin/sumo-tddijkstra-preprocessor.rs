use std::error::Error;
use std::path::Path;

use clap::Parser;
use conversion::sumo::sumo_to_td_graph_converter::convert_sumo_to_routing_kit_and_queries;
use fastdta::cli;
use fastdta::logger::Logger;
use rust_road_router::report::measure;

/// has the following parameters:
/// - input_dir: the directory containing the input files
/// - input_prefix: the prefix of the input files
/// - output_dir: the directory to write the output files to (optional, defaults to current directory)
/// - seed: the random seed to use for the inertial flow cutter (optional, defaults to 5489)
/// - routing_threads: the number of threads to use for the routing
fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::PreprocesserArgs::parse();

    let input_dir = Path::new(&args.input_dir);
    let input_prefix = args.input_prefix;
    let trips_file = Path::new(&args.trips_file);
    let output_dir = Path::new(&args.output_dir);

    let begin = args.begin;
    let end = args.end;
    let interval = args.interval;

    let logger = Logger::new("sumo-tddijkstra-preprocessor", &input_dir.display().to_string(), -1);

    let (_, duration) = measure(|| convert_sumo_to_routing_kit_and_queries(&input_dir, &input_prefix, &trips_file, &output_dir, begin, end, interval));
    logger.log("preprocessing", duration.as_nanos());

    Ok(())
}
