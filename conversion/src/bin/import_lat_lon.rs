/// reads a file containing information on a node's latitude and longitute information and writes two binary files `latitude` and `longitude`
/// containing f32 floating point numbers into a given output directory
/// The input file has the following format:
/// For each node:
///     <latitude> <longitude>\n
///
/// We use this converter to we can add a human-readable file as an input and continue working with the binary files `latitude` and `longitude`
/// as an input for InertialFlowCutter
use rust_road_router::{cli::CliErr, io::*};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);

    let input = &args.next().ok_or(CliErr("No input file given"))?;
    let input = Path::new(input);

    let output = &args.next().ok_or(CliErr("No output dir given"))?;
    let out_dir = Path::new(output);

    let input = File::open(input)?;
    let mut lines = BufReader::new(&input).lines();

    let mut lat: Vec<f32> = Vec::new();
    let mut lon: Vec<f32> = Vec::new();

    while let Some(Ok(line)) = lines.next() {
        let mut words = line.split_whitespace();
        lat.push(words.next().unwrap().parse::<f32>().unwrap());
        lon.push(words.next().unwrap().parse::<f32>().unwrap());
    }

    lat.write_to(&out_dir.join("latitude"))?;
    lon.write_to(&out_dir.join("longitude"))?;

    Ok(())
}
