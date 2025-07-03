use std::error::Error;
use std::path::Path;

use conversion::sumo::sumo_to_td_graph_converter::{convert_sumo_to_td_graph, read_nodes_edges_and_trips_from_plain_xml};

use fastdta::cli;
use fastdta::cli::Parser;
use rust_road_router::datastr::graph::Graph;
use rust_road_router::io::Store;

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::Args::parse();

    let Some(input_dir) = args.input_dir else {
        panic!("No input directory provided to read files from. Use --input-dir <path> to specify a directory containing all of the input files.");
    };

    let Some(input_prefix) = args.input_prefix else {
        panic!("No input prefix provided. Use --input-prefix <prefix> (or -i <prefix>) to specify the prefix of each input file.");
    };

    let Some(output_dir) = args.output_dir else {
        panic!("No output directory provided. Use --output-dir <path> to specify an output directory for the TD-CCH.");
    };

    let input_dir = Path::new(&input_dir);
    let output_dir = Path::new(&output_dir);

    let (nodes, edges, trips) = read_nodes_edges_and_trips_from_plain_xml(input_dir, &input_prefix);

    let (g, edges_by_id) = convert_sumo_to_td_graph(&nodes, &edges);

    let (lat, lon) = nodes.get_latitude_longitude();

    // necessary for creating the TD-CCH
    lat.write_to(&output_dir.join("latitude"))?;
    lon.write_to(&output_dir.join("longitude"))?;
    g.0.write_to(&output_dir.join("first_out"))?;
    g.1.write_to(&output_dir.join("head"))?;
    g.2.write_to(&output_dir.join("first_ipp_of_arc"))?;
    g.3.write_to(&output_dir.join("ipp_departure_time"))?;
    g.4.write_to(&output_dir.join("ipp_travel_time"))?;

    edges_by_id.write_to(&output_dir.join("edges_by_id"))?;

    // create a subprocess which runs the bash script: "flow_cutter_cch_cut_order.sh <output_dir>" to create node rankings for the TD-CCH

    let output_dir_str = output_dir.to_str().ok_or("Output directory is not valid UTF-8")?;
    let status = std::process::Command::new("bash")
        .arg("flow_cutter_cch_cut_order.sh")
        .arg(output_dir_str)
        .current_dir(dir)
        .status()?;

    if !status.success() {
        return Err(Box::new(cli::CliErr("Failed to run flow_cutter_cch_cut_order.sh script".to_string())));
    }

    Ok(())
}
