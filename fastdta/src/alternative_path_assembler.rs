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

// assemble the alternatives
// merge previous alternatives with current paths (make sure there are no duplicates)
// calculate costs for each path in the current graph
// choose a path based on the choice algorithm
// return alternative paths, choice, probabilities and costs

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
            alternatives: shortest_paths
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
        };

        alternative_paths.deconstruct(&current_iteration_dir.join(DIR_DTA)).unwrap();
    };

    write_paths_as_sumo_routes(&input_dir, &input_prefix, iteration, &shortest_paths, &departures, &edge_indices_to_id);
}

/// merges the previous alternatives with the current shortest paths
/// calculates the travel times for each alternative path using the edge weights from the current graph
/// update the `alternative_paths` with the new paths and their travel times
fn merge_alternative_paths_with_new_travel_times(
    alternative_paths: &AlternativePathsForDTA,
    shortest_paths: &Vec<Vec<u32>>,
    travel_times: &Vec<FlWeight>,
    departures: &Vec<SerializedTimestamp>,
    graph: &TDGraph,
) -> AlternativePathsForDTA {
    let mut merged_alternative_paths = alternative_paths.clone();
    // merge previous alternatives with current paths
    for (i, alternatives) in merged_alternative_paths.alternatives.iter_mut().enumerate() {
        let mut is_shortest_path_among_alternatives = false;
        for (j, alternative_path) in alternatives.paths.iter().enumerate() {
            if alternative_path.edges == *shortest_paths[i] {
                // path already exists, skip it
                is_shortest_path_among_alternatives = true;
                alternatives.costs[j] = travel_times[i].into();
            } else {
                alternatives.costs[j] = graph
                    .get_travel_time_along_path(Timestamp::from_millis(departures[i]), &alternative_path.edges)
                    .into();
            }
        }

        if !is_shortest_path_among_alternatives {
            alternatives.paths.push(AlternativePath {
                edges: shortest_paths[i].clone(),
            });
            alternatives.costs.push(travel_times[i].into());
        }
    }

    merged_alternative_paths
}
