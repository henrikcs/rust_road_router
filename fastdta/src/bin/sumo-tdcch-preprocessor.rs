use std::env;
use std::error::Error;
use std::path::Path;

use conversion::sumo::sumo_to_td_graph_converter::{
    convert_sumo_to_routing_kit, get_routing_kit_td_graph_from_sumo, read_nodes_edges_and_trips_from_plain_xml,
};

use fastdta::cli;
use fastdta::cli::Parser;
use rust_road_router::algo::customizable_contraction_hierarchy::{CCHT, contract, reorder, reorder_for_seperator_based_customization};
use rust_road_router::datastr::graph::UnweightedOwnedGraph;
use rust_road_router::datastr::node_order::NodeOrder;
use rust_road_router::io::{Deconstruct, Load, Reconstruct};

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::Args::parse();

    let Some(input_dir) = args.input_dir else {
        panic!("No input directory provided to read files from. Use --input-dir <path> to specify a directory containing all of the input files.");
    };

    let Some(input_prefix) = args.input_prefix else {
        panic!("No input prefix provided. Use --input-prefix <prefix> (or -i <prefix>) to specify the prefix of each input file.");
    };

    let output_dir = args.output_dir.unwrap_or(String::from(env::current_dir()?.to_str().unwrap()));
    let input_dir = Path::new(&input_dir);
    let output_dir = Path::new(&output_dir);
    dbg!(&output_dir);

    convert_sumo_to_routing_kit(&input_dir, &input_prefix, &output_dir)?;

    dbg!(&output_dir);

    // create a subprocess which runs the bash script: "flow_cutter_cch_cut_order.sh <output_dir>" to create node rankings for the TD-CCH
    run_inertial_flow_cutter(
        &output_dir,
        args.seed.unwrap_or(5489),
        args.routing_threads.unwrap_or(std::thread::available_parallelism().unwrap().get() as i32),
    )?;

    dbg!(&output_dir);
    // run catchup proprocessing
    preprocess(&output_dir)?;

    Ok(())
}

fn preprocess(working_dir: &Path) -> Result<(), Box<dyn Error>> {
    // TODO: instead of reading from files, we should have parameters passed to the function; evaluate the memory usage
    let graph = UnweightedOwnedGraph::reconstruct_from(&working_dir)?;

    // let mut algo_runs_ctxt = push_collection_context("algo_runs");

    let cch_folder = working_dir.join("cch");

    let cch_order = NodeOrder::from_node_order(Vec::load_from(working_dir.join("cch_perm"))?);
    // let cch_build_ctxt = algo_runs_ctxt.push_collection_item();
    let cch = contract(&graph, cch_order);
    // drop(cch_build_ctxt);

    let latitude = Vec::<f32>::load_from(working_dir.join("latitude"))?;
    let longitude = Vec::<f32>::load_from(working_dir.join("longitude"))?;

    let cch_order = reorder(&cch, &latitude, &longitude);

    // let cch_build_ctxt = algo_runs_ctxt.push_collection_item();
    let cch = contract(&graph, cch_order.clone());
    // drop(cch_build_ctxt);

    // TODO optimize away the clone
    let cch_order = reorder_for_seperator_based_customization(&cch_order, cch.separators().clone());
    cch_order.deconstruct_to(&cch_folder)?;

    // let cch_build_ctxt = algo_runs_ctxt.push_collection_item();
    let cch = contract(&graph, cch_order);
    // drop(cch_build_ctxt);

    cch.deconstruct_to(&cch_folder)?;

    Ok(())
}

fn run_inertial_flow_cutter(directory: &Path, seed: i32, threads: i32) -> Result<(), Box<dyn Error>> {
    // make sure that "console" is in the PATH (i.e. lib/InertialFlowCutter/build/console)
    // the values have been copied from flow_cutter_cch_order.sh:

    // load_routingkit_unweighted_graph "$1/first_out" "$1/head" \
    // load_routingkit_longitude "$1/longitude" \
    // load_routingkit_latitude "$1/latitude" \
    // remove_multi_arcs \
    // remove_loops \
    // add_back_arcs \
    // sort_arcs \
    // flow_cutter_set random_seed $seed \
    // reorder_nodes_at_random \
    // reorder_nodes_in_preorder \
    // flow_cutter_set thread_count ${2:--1} \
    // flow_cutter_set BulkDistance no \
    // flow_cutter_set max_cut_size 100000000 \
    // flow_cutter_set distance_ordering_cutter_count 0 \
    // flow_cutter_set geo_pos_ordering_cutter_count 8 \
    // flow_cutter_set bulk_assimilation_threshold 0.4 \
    // flow_cutter_set bulk_assimilation_order_threshold 0.25 \
    // flow_cutter_set bulk_step_fraction 0.05 \
    // flow_cutter_set initial_assimilated_fraction 0.05 \
    // flow_cutter_config \
    // report_time \
    // reorder_nodes_in_accelerated_flow_cutter_cch_order \
    // do_not_report_time \
    // examine_chordal_supergraph \
    // save_routingkit_node_permutation_since_last_load "$1/cch_perm"

    dbg!(&directory);
    let status = std::process::Command::new("console")
        .arg("load_routingkit_unweighted_graph")
        .arg(directory.join("first_out").to_str().unwrap())
        .arg(directory.join("head").to_str().unwrap())
        .arg("load_routingkit_longitude")
        .arg(directory.join("longitude").to_str().unwrap())
        .arg("load_routingkit_latitude")
        .arg(directory.join("latitude").to_str().unwrap())
        .arg("remove_multi_arcs")
        .arg("remove_loops")
        .arg("add_back_arcs")
        .arg("sort_arcs")
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
        .arg("reorder_nodes_in_accelerated_flow_cutter_cch_order")
        .arg("do_not_report_time")
        .arg("examine_chordal_supergraph")
        .arg("save_routingkit_node_permutation_since_last_load")
        .arg(directory.join("cch_perm").to_str().unwrap())
        .status()?;

    if !status.success() {
        return Err(Box::from("Failed to run Inertial Flow Cutter console command"));
    }

    Ok(())
}
