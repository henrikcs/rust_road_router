use std::error::Error;
use std::path::Path;

use conversion::{DIR_CCH, FILE_CCH_PERM, FILE_FIRST_OUT, FILE_HEAD, FILE_LATITUDE, FILE_LONGITUDE, SerializedPosition};
use rust_road_router::algo::customizable_contraction_hierarchy::{CCH, CCHReconstrctor, CCHT, contract, reorder, reorder_for_seperator_based_customization};
use rust_road_router::datastr::graph::UnweightedOwnedGraph;
use rust_road_router::datastr::graph::floating_time_dependent::TDGraph;
use rust_road_router::datastr::node_order::NodeOrder;
use rust_road_router::io::{Deconstruct, Load, Reconstruct, ReconstructPrepared};

pub fn preprocess(working_dir: &Path) -> Result<(), Box<dyn Error>> {
    // TODO: instead of reading from files, we should have parameters passed to the function; evaluate the memory usage
    // use first_out and head to reconstruct the graph
    let graph = UnweightedOwnedGraph::reconstruct_from(&working_dir)?;

    // let mut algo_runs_ctxt = push_collection_context("algo_runs");

    let cch_folder = working_dir.join(DIR_CCH);

    let cch_order = NodeOrder::from_node_order(Vec::load_from(working_dir.join(FILE_CCH_PERM))?);
    // let cch_build_ctxt = algo_runs_ctxt.push_collection_item();
    let cch = contract(&graph, cch_order);
    // drop(cch_build_ctxt);

    let latitude = Vec::<SerializedPosition>::load_from(working_dir.join(FILE_LATITUDE))?;
    let longitude = Vec::<SerializedPosition>::load_from(working_dir.join(FILE_LONGITUDE))?;

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

pub fn get_cch(input_dir: &Path, graph: &TDGraph) -> CCH {
    let cch_folder = input_dir.join(DIR_CCH);
    // TODO: instead of reconstructing the graph from disk, we could create it in memory
    println!("Reconstructing CCH from: {}", cch_folder.display());
    CCHReconstrctor(graph).reconstruct_from(&cch_folder).unwrap()
}

pub fn run_inertial_flow_cutter(directory: &Path, seed: i32, threads: i32) -> Result<(), Box<dyn Error>> {
    // make sure that "console" is in the PATH (i.e. lib/InertialFlowCutter/build/console)
    // the values have been copied from flow_cutter_cch_order.sh:
    let status = std::process::Command::new("console")
        .arg("load_routingkit_unweighted_graph")
        .arg(directory.join(FILE_FIRST_OUT).to_str().unwrap())
        .arg(directory.join(FILE_HEAD).to_str().unwrap())
        .arg("load_routingkit_longitude")
        .arg(directory.join(FILE_LONGITUDE).to_str().unwrap())
        .arg("load_routingkit_latitude")
        .arg(directory.join(FILE_LATITUDE).to_str().unwrap())
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
        .arg(directory.join(FILE_CCH_PERM).to_str().unwrap())
        .status()?;

    if !status.success() {
        return Err(Box::from("Failed to run Inertial Flow Cutter console command"));
    }

    Ok(())
}
