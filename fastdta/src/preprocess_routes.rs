use std::path::Path;

use conversion::{
    DIR_DTA, FILE_EDGE_INDICES_TO_ID, FILE_EDGE_SPEEDS, GLOBAL_FREE_FLOW_SPEED_FACTOR,
    sumo::{
        FileReader, meandata::MeandataDocumentRoot, meandata_reader::SumoMeandataReader, sumo_find_file::get_meandata_file,
        sumo_to_new_graph_weights::get_graph_with_travel_times_from_previous_iteration,
    },
};
use rand::{Rng, SeedableRng, rngs};
use rust_road_router::{
    algo::customizable_contraction_hierarchy::CCH,
    datastr::graph::floating_time_dependent::{FlWeight, TDGraph},
    io::{Load, read_strings_from_file},
};

use crate::{
    alternative_paths::AlternativePathsForDTA, calculate_keep_routes, preprocess::get_cch, query::read_queries, traffic_model::TrafficModelType,
    traffic_model_data::TrafficModelData,
};

pub fn get_graph_data_for_fast_dta(
    input_dir: &Path,
    iteration: u32,
    traffic_model_type: TrafficModelType,
    keep_route_probability: f64,
) -> (
    Vec<String>,
    (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>),
    MeandataDocumentRoot,
    AlternativePathsForDTA,
    TrafficModelData,
    Vec<bool>,
) {
    let edge_ids = get_edge_ids(input_dir);
    let query_data = read_queries(input_dir);

    // TODO: in `get_graph_with_travel_times_from_previous_iteration` we already read meandata; reuse it here
    let meandata = if iteration > 0 {
        let iteration_dir = input_dir.join(format!("{:0>3}", iteration - 1));

        SumoMeandataReader::read(&get_meandata_file(&iteration_dir)).expect("Failed to read SUMO meandata")
    } else {
        MeandataDocumentRoot::empty()
    };
    let number_of_queries = query_data.0.len();

    if iteration == 0 {
        let free_flow_speeds: Vec<f64> = Vec::<f64>::load_from(&input_dir.join(FILE_EDGE_SPEEDS))
            .unwrap()
            .iter()
            .map(|ffs| *ffs * 3.6 * GLOBAL_FREE_FLOW_SPEED_FACTOR)
            .collect();

        return (
            edge_ids,
            query_data,
            MeandataDocumentRoot::empty(),
            AlternativePathsForDTA::init(&vec![vec![]; number_of_queries], &vec![FlWeight::new(0.0); number_of_queries]),
            TrafficModelData::init(&free_flow_speeds, TrafficModelType::ModifiedLee),
            vec![false; number_of_queries],
        );
    }

    let iteration_dir = input_dir.join(format!("{:0>3}", iteration - 1));

    let alternative_paths = AlternativePathsForDTA::reconstruct(&iteration_dir.join(DIR_DTA));
    // traffic model data might be empty if no calibration was done before
    let traffic_model_data = TrafficModelData::reconstruct(&input_dir, traffic_model_type);

    (
        edge_ids,
        query_data,
        meandata,
        alternative_paths,
        traffic_model_data,
        calculate_keep_routes(number_of_queries, keep_route_probability, rand::random::<i32>()),
    )
}

pub fn get_graph_data_for_cch(input_dir: &Path, iteration: u32) -> (Vec<String>, TDGraph, CCH) {
    let (edge_ids, graph) = get_graph_data_for_dijkstra(input_dir, iteration);
    let cch = get_cch(input_dir, &graph);

    (edge_ids, graph, cch)
}

pub fn get_graph_data_for_dijkstra(input_dir: &Path, iteration: u32) -> (Vec<String>, TDGraph) {
    let edge_ids: Vec<String> = get_edge_ids(input_dir);
    let graph = get_graph_with_travel_times_from_previous_iteration(input_dir, iteration, &edge_ids);

    (edge_ids, graph)
}

fn get_edge_ids(input_dir: &Path) -> Vec<String> {
    read_strings_from_file(&input_dir.join(FILE_EDGE_INDICES_TO_ID)).unwrap_or_else(|_| {
        panic!(
            "Failed to read edge indices from file {} in directory {}",
            FILE_EDGE_INDICES_TO_ID,
            input_dir.display()
        )
    })
}
