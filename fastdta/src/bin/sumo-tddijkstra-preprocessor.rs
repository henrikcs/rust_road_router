use std::error::Error;
use std::path::Path;

use clap::Parser;
use conversion::sumo::sumo_to_td_graph_converter::convert_sumo_to_routing_kit_and_queries;
use fastdta::cli;
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
    let output_dir = Path::new(&args.output_dir);

    println!("Input directory: {}", input_dir.display());
    println!("Input prefix: {}", input_prefix);
    println!("Output directory: {}", output_dir.display());

    let (_, duration) = measure(|| convert_sumo_to_routing_kit_and_queries(&input_dir, &input_prefix, &output_dir));
    log(&output_dir.display().to_string(), "preprocessing", duration.as_nanos());

    Ok(())
}

/// Logs the operation with the duration in nanoseconds within a certain iteration of certain run identified by identifier.
/// The format is: "sumo-tdcch-preprocessor; <identifier>; <iteration>; <operation>; <duration_in_nanos>"
fn log(identifier: &str, operation: &str, duration_in_nanos: u128) {
    println!("sumo-tddijkstra-preprocessor; {}; -1; {}; {}", identifier, operation, duration_in_nanos);
}
