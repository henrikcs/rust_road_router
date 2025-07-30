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
    let alternative_paths = if iteration > 0 {
        // load previous alternatives from input_dir
        println!("Loading previous alternatives from iteration {}", iteration - 1);
        let previous_iteration_dir = input_dir.join(format!("{:0>3}", iteration - 1));
        let old_alternative_paths: AlternativePathsForDTA = AlternativePathsForDTA::reconstruct(&previous_iteration_dir.join(DIR_DTA));

        println!("Merging previous alternatives with current shortest paths");
        // merge previous alternatives with current shortest paths
        let mut new_alternative_paths = old_alternative_paths.update_alternatives_with_new_paths(&shortest_paths, &travel_times, &departures, &graph);

        println!("Performing choice model on the new alternatives");
        new_alternative_paths.perform_choice_model(&old_alternative_paths, &choice_algorithm, max_alternatives, seed);

        new_alternative_paths
    } else {
        // initialize with alternatives consisting of the shortest paths
        AlternativePathsForDTA::init(shortest_paths, travel_times)
    };

    println!("Assembling alternative paths for DTA with {} alternatives", max_alternatives);
    let (path_sets, costs, probabilities, choices) = transform_alternative_paths_for_dta_to_vectors(&alternative_paths);

    println!("Writing alternative paths to SUMO routes");
    write_paths_as_sumo_routes(
        &input_dir,
        &input_prefix,
        iteration,
        &path_sets,
        &costs,
        &probabilities,
        &choices,
        &departures,
        &edge_indices_to_id,
    );

    println!("Serializing alternative paths to DTA format");
    alternative_paths.deconstruct(&current_iteration_dir.join(DIR_DTA)).unwrap();
}

fn transform_alternative_paths_for_dta_to_vectors(
    alternative_paths: &AlternativePathsForDTA,
) -> (Vec<Vec<Vec<u32>>>, Vec<Vec<f64>>, Vec<Vec<f64>>, Vec<usize>) {
    let mut path_sets = vec![vec![]; alternative_paths.alternatives_in_query.len()];
    let mut costs = vec![vec![]; alternative_paths.alternatives_in_query.len()];
    let mut probabilities = vec![vec![]; alternative_paths.alternatives_in_query.len()];
    let mut choices = vec![0 as usize; alternative_paths.alternatives_in_query.len()];

    for (i, alternatives) in alternative_paths.alternatives_in_query.iter().enumerate() {
        let mut path_set = vec![];
        for path in alternatives.paths.iter() {
            // add the path to the path set
            path_set.push(path.edges.clone());
        }
        path_sets[i].append(&mut path_set);
        costs[i].append(&mut alternatives.costs.clone());
        probabilities[i].append(&mut alternatives.probabilities.clone());
        choices[i] = alternatives.choice;
    }

    (path_sets, costs, probabilities, choices)
}
