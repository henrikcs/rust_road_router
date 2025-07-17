use std::path::Path;

use conversion::{DIR_DTA, SerializedTimestamp, sumo::paths_to_sumo_routes_converter::write_paths_as_sumo_routes};
use rust_road_router::datastr::graph::{
    EdgeId,
    floating_time_dependent::{FlWeight, TDGraph, Timestamp},
};

use crate::{
    choice::ChoiceAlgorithm,
    dta_alternative_paths::{AlternativePath, AlternativePaths, AlternativePathsForDTA},
};

/// assemble the alternatives
/// merge previous alternatives with current paths (make sure there are no duplicates)
/// calculate costs for each path in the current graph
/// choose a path based on the choice algorithm
/// return alternative paths, choice, probabilities and costs
pub fn assemble_alternative_paths(
    input_dir: &Path,
    input_prefix: &String,
    iteration: u32,
    shortest_paths: &Vec<Vec<u32>>,
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
        let alternative_paths: AlternativePathsForDTA = AlternativePathsForDTA::reconstruct(&previous_iteration_dir.join(DIR_DTA));

        let mut new_alternative_paths = merge_alternative_paths_with_new_travel_times(&alternative_paths, &shortest_paths, &travel_times, &departures, &graph);

        new_alternative_paths.perform_choice_model(&alternative_paths, &choice_algorithm, max_alternatives, seed);
        new_alternative_paths.deconstruct(&current_iteration_dir.join(DIR_DTA)).unwrap();
    } else {
        // initialize with alternatives consisting of the shortest paths
        let alternative_paths = AlternativePathsForDTA {
            alternatives_in_query: shortest_paths
                .iter()
                .enumerate()
                .map(|(i, path)| AlternativePaths {
                    paths: vec![AlternativePath {
                        edges: path.iter().map(|&e| e as EdgeId).collect(),
                    }],
                    costs: vec![travel_times[i].into()],
                    probabilities: vec![1.0],
                    choice: 0,
                })
                .collect(),
            new_path_in_query: vec![true; shortest_paths.len()],
        };

        alternative_paths.deconstruct(&current_iteration_dir.join(DIR_DTA)).unwrap();
    };

    write_paths_as_sumo_routes(&input_dir, &input_prefix, iteration, &shortest_paths, &departures, &edge_indices_to_id);
}

/// merges the previous alternatives with the current shortest paths
/// adds new paths without computing travel times yet (will be done in perform_choice_model)
fn merge_alternative_paths_with_new_travel_times(
    alternative_paths: &AlternativePathsForDTA,
    shortest_paths: &Vec<Vec<u32>>,
    travel_times: &Vec<FlWeight>,
    departures: &Vec<SerializedTimestamp>,
    graph: &TDGraph,
) -> AlternativePathsForDTA {
    let mut merged_alternative_paths = alternative_paths.clone();

    // Ensure new_path_in_query has the right size
    if merged_alternative_paths.new_path_in_query.len() != shortest_paths.len() {
        merged_alternative_paths.new_path_in_query.resize(shortest_paths.len(), false);
    }

    // merge previous alternatives with current paths
    for (i, alternatives) in merged_alternative_paths.alternatives_in_query.iter_mut().enumerate() {
        let mut is_shortest_path_among_alternatives = false;
        merged_alternative_paths.new_path_in_query[i] = false;

        // Update costs for existing paths
        for (j, alternative_path) in alternatives.paths.iter().enumerate() {
            if alternative_path.edges == shortest_paths[i].iter().map(|&e| e as EdgeId).collect::<Vec<EdgeId>>() {
                // path already exists, mark it and update its cost
                is_shortest_path_among_alternatives = true;
                alternatives.costs[j] = travel_times[i].into();
            } else {
                // Recompute cost for existing alternative path
                alternatives.costs[j] = graph
                    .get_travel_time_along_path(Timestamp::from_millis(departures[i]), &alternative_path.edges)
                    .into();
            }
        }

        // Add new shortest path if it's not among existing alternatives
        if !is_shortest_path_among_alternatives {
            alternatives.paths.push(AlternativePath {
                edges: shortest_paths[i].iter().map(|&e| e as EdgeId).collect(),
            });
            alternatives.costs.push(travel_times[i].into());

            // Extend probabilities vector with initial probability for new route
            alternatives.probabilities.push(1.0 / alternatives.paths.len() as f64);

            merged_alternative_paths.new_path_in_query[i] = true; // Mark that a new path was added
        }
    }

    merged_alternative_paths
}
