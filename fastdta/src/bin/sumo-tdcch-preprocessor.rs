use std::error::Error;
use std::path::Path;

use clap::Parser;
use conversion::sumo::sumo_to_td_graph_converter::convert_sumo_to_routing_kit_and_queries;
use fastdta::cli;
use fastdta::preprocess::{preprocess, run_inertial_flow_cutter};

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

    convert_sumo_to_routing_kit_and_queries(&input_dir, &input_prefix, &output_dir)?;
    // create a subprocess which runs the bash script: "flow_cutter_cch_cut_order.sh <output_dir>" to create node rankings for the TD-CCH
    run_inertial_flow_cutter(&output_dir, args.seed, args.routing_threads)?;
    // run catchup proprocessing
    preprocess(&output_dir)?;

    Ok(())
}
