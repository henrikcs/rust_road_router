use std::env;
use std::path::Path;

use conversion::sumo::sumo_to_td_graph_converter::{DIR_CCH, FILE_QUERIES_DEPARTURE, FILE_QUERIES_FROM, FILE_QUERIES_TO};
use fastdta::cli;
use fastdta::cli::Parser;
use rust_road_router::algo::catchup::Server;
use rust_road_router::algo::customizable_contraction_hierarchy::{CCHReconstrctor, ftd_cch};
use rust_road_router::algo::{TDQuery, TDQueryServer};
use rust_road_router::datastr::graph::floating_time_dependent::{FlWeight, TDGraph, Timestamp};
use rust_road_router::io::{Load, Reconstruct, ReconstructPrepared};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let _reporter = enable_reporting("tdcch_customization");
    // report!("num_threads", rayon::current_num_threads());

    let args = cli::Args::parse();

    let output_dir = args.output_dir.unwrap_or(String::from(env::current_dir()?.to_str().unwrap()));
    let output_dir = Path::new(&output_dir);

    let graph = TDGraph::reconstruct_from(&output_dir).expect("Failed to reconstruct the time-dependent graph");

    // let mut algo_runs_ctxt = push_collection_context("algo_runs");

    let cch_folder = output_dir.join(DIR_CCH);
    let cch = CCHReconstrctor(&graph).reconstruct_from(&cch_folder).unwrap();

    // let _cch_customization_ctxt = algo_runs_ctxt.push_collection_item();
    let customized_graph = ftd_cch::customize(&cch, &graph);

    // read queries from output_dir
    let queries_from = Vec::<u32>::load_from(output_dir.join(FILE_QUERIES_FROM)).unwrap();
    let queries_to = Vec::<u32>::load_from(output_dir.join(FILE_QUERIES_TO)).unwrap();
    let queries_departure = Vec::<f64>::load_from(output_dir.join(FILE_QUERIES_DEPARTURE)).unwrap();

    assert!(queries_from.len() == queries_to.len());
    assert!(queries_from.len() == queries_departure.len());

    let mut query_server = Server::new(&cch, &customized_graph);

    for i in 0..queries_from.len() {
        let result = query_server.td_query(TDQuery {
            from: queries_from[i] as u32,
            to: queries_to[i] as u32,
            departure: Timestamp::new(queries_departure[i]),
        });
        if let Some(result) = result.found() {
            let ea = FlWeight::new(queries_departure[i]) + result.distance();
            println!(
                "From: {}, To: {}, Departure: {}, Earliest Arrival: {:?}",
                queries_from[i], queries_to[i], queries_departure[i], ea
            );
        } else {
            println!("No path found from {} to {} at {}", queries_from[i], queries_to[i], queries_departure[i]);
        }
    }

    // TODO: write query results to a file

    Ok(())
}
