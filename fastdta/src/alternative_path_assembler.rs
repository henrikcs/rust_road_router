use std::path::Path;

use conversion::{DIR_DTA, SerializedTimestamp, sumo::paths_to_sumo_routes_converter::write_paths_as_sumo_routes};
use rust_road_router::datastr::graph::{
    EdgeId,
    floating_time_dependent::{FlWeight, TDGraph},
};

use crate::{alternative_paths::AlternativePathsForDTA, choice::ChoiceAlgorithm};

/// assemble the alternatives
/// merge previous alternatives with current paths (make sure there are no duplicates)
/// calculate costs for each path in the current graph
/// choose a path based on the choice algorithm
/// return alternative paths, choice, probabilities and costs
pub fn assemble_alternative_paths(
    input_dir: &Path,
    input_prefix: &String,
    iteration: u32,
    shortest_paths: &Vec<Vec<EdgeId>>,
    travel_times: &Vec<FlWeight>,
    departures: &Vec<SerializedTimestamp>,
    graph: &TDGraph,
    choice_algorithm: ChoiceAlgorithm,
    max_alternatives: u32,
    seed: i32,
    edge_indices_to_id: &Vec<String>,
) {
    let current_iteration_dir = input_dir.join(format!("{:0>3}", iteration));

    // init all_routes with the previous alternatives
    if iteration > 0 {
        // load previous alternatives from input_dir
        let previous_iteration_dir = input_dir.join(format!("{:0>3}", iteration - 1));
        let old_alternative_paths: AlternativePathsForDTA = AlternativePathsForDTA::reconstruct(&previous_iteration_dir.join(DIR_DTA));

        // merge previous alternatives with current shortest paths
        let mut new_alternative_paths = old_alternative_paths.update_alternatives_with_new_paths(&shortest_paths, &travel_times, &departures, &graph);

        new_alternative_paths.perform_choice_model(&old_alternative_paths, &choice_algorithm, max_alternatives, seed);

        new_alternative_paths.deconstruct(&current_iteration_dir.join(DIR_DTA)).unwrap();
    } else {
        // initialize with alternatives consisting of the shortest paths
        let alternative_paths = AlternativePathsForDTA::init(shortest_paths, travel_times);

        alternative_paths.deconstruct(&current_iteration_dir.join(DIR_DTA)).unwrap();
    };

    write_paths_as_sumo_routes(&input_dir, &input_prefix, iteration, &shortest_paths, &departures, &edge_indices_to_id);
}
