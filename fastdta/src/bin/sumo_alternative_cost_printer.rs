use std::{env, path::Path};

use clap::Parser;
use conversion::FILE_DTA_QUERIES_ALTERNATIVE_COST;
use rust_road_router::io::Load;

fn main() {
    let args = PreprocesserArgs::parse();

    // Print the input directory
    println!("Input directory: {}", args.input_dir);

    // Here you would typically call your preprocessing function with the input directory
    // preprocess(&args.input_dir);

    let dir = Path::new(&args.input_dir);

    let cost_file = dir.join(FILE_DTA_QUERIES_ALTERNATIVE_COST);

    // read all edges with sumoedgesreader
    let costs: Vec<f64> = Vec::load_from(&cost_file).unwrap();

    dbg!(costs);
}

/// Command-line arguments for fast-dta preprocessing
#[derive(Parser, Debug)]
#[command(version, about = "fast-dta preprocessing CLI options", long_about = None)]
pub struct PreprocesserArgs {
    /// the directory containing the input files
    #[arg(long = "input-dir", default_value_t = String::from(env::current_dir().unwrap().to_str().unwrap()))]
    pub input_dir: String,
}
