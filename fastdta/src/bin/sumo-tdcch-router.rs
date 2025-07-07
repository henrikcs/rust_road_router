use std::fs;
use std::path::Path;

use conversion::sumo::ROUTES;
use conversion::sumo::paths_to_sumo_routes_converter::write_paths_as_sumo_routes;
use conversion::sumo::sumo_to_new_graph_weights::set_new_graph_weights_from_meandata_file;
use conversion::{
    DIR_CCH, FILE_EDGE_DEFAULT_TRAVEL_TIMES, FILE_EDGE_INDICES_TO_ID, FILE_QUERIES_DEPARTURE, FILE_QUERIES_FROM, FILE_QUERIES_TO, FILE_QUERY_IDS,
    FILE_QUERY_ORIGINAL_FROM_EDGES, FILE_QUERY_ORIGINAL_TO_EDGES,
};
use fastdta::cli;
use fastdta::cli::Parser;
use rust_road_router::algo::catchup::Server;
use rust_road_router::algo::customizable_contraction_hierarchy::{CCHReconstrctor, ftd_cch};
use rust_road_router::algo::{PathServer, TDQuery, TDQueryServer};
use rust_road_router::datastr::graph::floating_time_dependent::{TDGraph, Timestamp};
use rust_road_router::io::{Load, Reconstruct, ReconstructPrepared, read_strings_from_file};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let _reporter = enable_reporting("tdcch_customization");
    // report!("num_threads", rayon::current_num_threads());

    let args = cli::RouterArgs::parse();

    let input_dir = Path::new(&args.input_dir);
    let input_prefix = args.input_prefix;
    let iteration = args.iteration;

    let current_iteration_dir = input_dir.join(format!("{iteration:0>3}"));

    let graph = TDGraph::reconstruct_from(&input_dir).expect("Failed to reconstruct the time-dependent graph");
    let edge_ids: Vec<String> = read_strings_from_file(&input_dir.join(FILE_EDGE_INDICES_TO_ID)).unwrap();

    // if iteration > 0, we load the previous iteration's travel times
    if iteration > 0 {
        let previous_iteration_dir = input_dir.join(format!("{:0>3}", iteration - 1));

        // dump file starts with "dump_" and ends with ".xml"
        let dump_file = fs::read_dir(previous_iteration_dir)
            .unwrap()
            .find(|entry| {
                // check if entry is a file
                entry.is_ok()
                    && entry.as_ref().unwrap().file_type().unwrap().is_file()
                    && entry
                        .as_ref()
                        .unwrap()
                        .file_name()
                        .to_str()
                        .unwrap()
                        // check if file name starts with "dump_" and ends with ".xml"
                        .starts_with("dump_")
                    && entry.as_ref().unwrap().file_name().to_str().unwrap().ends_with(".xml")
            })
            .map(|entry| entry.unwrap().path())
            .unwrap();

        set_new_graph_weights_from_meandata_file(
            &input_dir,
            &dump_file,
            &edge_ids,
            &Vec::<f64>::load_from(input_dir.join(FILE_EDGE_DEFAULT_TRAVEL_TIMES)).unwrap(),
        );
    }

    // let mut algo_runs_ctxt = push_collection_context("algo_runs");

    let cch_folder = input_dir.join(DIR_CCH);
    let cch = CCHReconstrctor(&graph).reconstruct_from(&cch_folder).unwrap();

    // let _cch_customization_ctxt = algo_runs_ctxt.push_collection_item();
    // customize the cch with the given graph having new travel time functions for each edge
    let customized_graph = ftd_cch::customize(&cch, &graph);

    // read queries from input_dir
    let queries_from = Vec::<u32>::load_from(input_dir.join(FILE_QUERIES_FROM)).unwrap();
    let queries_to = Vec::<u32>::load_from(input_dir.join(FILE_QUERIES_TO)).unwrap();
    let queries_departure = Vec::<f64>::load_from(input_dir.join(FILE_QUERIES_DEPARTURE)).unwrap();

    assert!(queries_from.len() == queries_to.len());
    assert!(queries_from.len() == queries_departure.len());

    let mut query_server = Server::new(&cch, &customized_graph);

    let mut paths = Vec::new();

    for i in 0..queries_from.len() {
        println!(
            "Find Earliest Arrival #{i} From: {}, To: {}, Departure: {}",
            queries_from[i], queries_to[i], queries_departure[i]
        );
        let result = query_server.td_query(TDQuery {
            from: queries_from[i] as u32,
            to: queries_to[i] as u32,
            departure: Timestamp::new(queries_departure[i]),
        });
        if let Some(mut result) = result.found() {
            let ea = result.distance();
            paths.push(result.data().reconstruct_edge_path().iter().map(|edge| edge.0).collect());

            println!(
                "From: {}, To: {}, Departure: {}, Earliest Arrival: {:?}",
                queries_from[i], queries_to[i], queries_departure[i], ea
            );
        } else {
            println!("No path found from {} to {} at {}", queries_from[i], queries_to[i], queries_departure[i]);
        }
    }

    let trip_ids: Vec<String> = read_strings_from_file(&input_dir.join(FILE_QUERY_IDS)).unwrap();
    let original_from_edges: Vec<String> = read_strings_from_file(&input_dir.join(FILE_QUERY_ORIGINAL_FROM_EDGES)).unwrap();
    let original_to_edges: Vec<String> = read_strings_from_file(&input_dir.join(FILE_QUERY_ORIGINAL_TO_EDGES)).unwrap();

    write_paths_as_sumo_routes(
        &current_iteration_dir.join(format!("{input_prefix}_{iteration:0>3}{ROUTES}")),
        &paths,
        &edge_ids,
        &trip_ids,
        &original_from_edges,
        &original_to_edges,
        &queries_departure,
    );

    Ok(())
}
