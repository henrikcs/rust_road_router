use std::error::Error;
use std::path::Path;

use clap::Parser;
use conversion::sumo::sumo_to_td_graph_converter::convert_sumo_to_routing_kit_and_queries;
use fastdta::cli;
use fastdta::preprocess::{preprocess, run_inertial_flow_cutter};
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

    let (_, duration) = measure(|| convert_sumo_to_routing_kit_and_queries(&input_dir, &input_prefix, &output_dir));
    log(&output_dir.display().to_string(), "preprocessing", duration.as_nanos());

    // create a subprocess which runs the bash script: "flow_cutter_cch_cut_order.sh <output_dir>" to create node rankings for the TD-CCH
    let (_, duration) = measure(|| run_inertial_flow_cutter(&output_dir, args.seed, args.routing_threads));
    log(&output_dir.display().to_string(), "inertial flow cutter", duration.as_nanos());

    // run catchup preprocessing
    let (_, duration) = measure(|| preprocess(&output_dir));
    log(&output_dir.display().to_string(), "cch preprocessing", duration.as_nanos());

    Ok(())
}

fn log(directory: &str, operation: &str, duration_in_nanos: u128) {
    println!("sumo-tdcch-preprocessor; {}; -1; {}; {}", directory, operation, duration_in_nanos);
}
