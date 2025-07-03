use std::env;
use std::error::Error;
use std::path::Path;

use conversion::sumo::sumo_to_td_graph_converter::{convert_sumo_to_td_graph, read_nodes_edges_and_trips_from_plain_xml};

use fastdta::cli;
use fastdta::cli::Parser;
use rust_road_router::io::Store;

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::Args::parse();

    let Some(input_dir) = args.input_dir else {
        panic!("No input directory provided to read files from. Use --input-dir <path> to specify a directory containing all of the input files.");
    };

    let Some(input_prefix) = args.input_prefix else {
        panic!("No input prefix provided. Use --input-prefix <prefix> (or -i <prefix>) to specify the prefix of each input file.");
    };

    let output_dir = args.output_dir.unwrap_or(String::from(env::current_dir()?.to_str().unwrap()));

    dbg!(&input_dir);
    dbg!(&input_prefix);
    dbg!(&output_dir);

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

    run_inertial_flow_cutter_console(output_dir, args.seed.unwrap_or(5489), args.routing_threads.unwrap_or(-1))?;

    Ok(())
}

fn run_inertial_flow_cutter_console(directory: &Path, seed: i32, threads: i32) -> Result<(), Box<dyn Error>> {
    // make sure that "console" is in the PATH (i.e. lib/InertialFlowCutter/build/console)
    // the values have been copied from flow_cutter_cch_cut_order.sh

    let status = std::process::Command::new("console")
        .arg("load_routingkit_unweighted_graph")
        .arg(directory.join("first_out").to_str().unwrap())
        .arg(directory.join("head").to_str().unwrap())
        .arg("load_routingkit_longitude")
        .arg(directory.join("longitude").to_str().unwrap())
        .arg("load_routingkit_latitude")
        .arg(directory.join("latitude").to_str().unwrap())
        .arg("flow_cutter_set")
        .arg("random_seed")
        .arg(seed.to_string())
        .arg("reorder_nodes_at_random")
        .arg("reorder_nodes_in_preorder")
        .arg("flow_cutter_set")
        .arg("thread_count")
        .arg(threads.to_string())
        .arg("flow_cutter_set")
        .arg("BulkDistance")
        .arg("no")
        .arg("flow_cutter_set")
        .arg("max_cut_size")
        .arg("100000000")
        .arg("flow_cutter_set")
        .arg("distance_ordering_cutter_count")
        .arg("0")
        .arg("flow_cutter_set")
        .arg("geo_pos_ordering_cutter_count")
        .arg("8")
        .arg("flow_cutter_set")
        .arg("bulk_assimilation_threshold")
        .arg("0.4")
        .arg("flow_cutter_set")
        .arg("bulk_assimilation_order_threshold")
        .arg("0.25")
        .arg("flow_cutter_set")
        .arg("bulk_step_fraction")
        .arg("0.05")
        .arg("flow_cutter_set")
        .arg("initial_assimilated_fraction")
        .arg("0.05")
        .arg("flow_cutter_config")
        .arg("report_time")
        .arg("reorder_arcs_in_accelerated_flow_cutter_cch_order")
        .arg("normal")
        .arg("do_not_report_time")
        .arg("save_routingkit_arc_permutation_since_last_load")
        .arg(directory.join("cch_perm_cuts").to_str().unwrap())
        .status()?;

    if !status.success() {
        return Err(Box::from("Failed to run Inertial Flow Cutter console command"));
    }

    Ok(())
}
