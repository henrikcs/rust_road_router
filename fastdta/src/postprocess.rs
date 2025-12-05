use std::path::Path;

use conversion::{DIR_DTA, SerializedTimestamp, sumo::paths_to_sumo_routes_converter::write_paths_as_sumo_routes};
use rand::{Rng, SeedableRng, rngs};
use rust_road_router::datastr::graph::{
    EdgeId,
    floating_time_dependent::{FlWeight, TDGraph, Timestamp},
};

use crate::{
    alternative_paths::AlternativePathsForDTA,
    choice::ChoiceAlgorithm,
    relative_gap::{append_relative_gap_to_file, get_relative_gap},
};

pub fn prepare_next_iteration_for_sampled_routing(
    input_dir: &Path,
    input_prefix: &String,
    iteration: u32,
    shortest_paths: &Vec<Vec<EdgeId>>,
    travel_times: &Vec<FlWeight>,
    departures: &Vec<SerializedTimestamp>,
    graph: &TDGraph,
    choice_algorithm: ChoiceAlgorithm,
    max_alternatives: u32,
    write_sumo_alternatives: bool,
    seed: i32,
    edge_indices_to_id: &Vec<String>,
    keep_routes: &Vec<bool>,
) {
    postprocess(
        input_dir,
        input_prefix,
        iteration,
        shortest_paths,
        travel_times,
        departures,
        graph,
        choice_algorithm,
        max_alternatives,
        write_sumo_alternatives,
        seed,
        edge_indices_to_id,
        keep_routes,
        true,
    );
}

/// calculate the relative gap,
/// assemble the alternative paths,
/// output as a sumo format for the next iteration
///
/// merge previous alternatives with current paths
/// calculate costs for each path in the current graph
/// choose a path based on the choice algorithm
/// return alternative paths, choice, probabilities and costs
pub fn prepare_next_iteration(
    input_dir: &Path,
    input_prefix: &String,
    iteration: u32,
    shortest_paths: &Vec<Vec<EdgeId>>,
    travel_times: &Vec<FlWeight>,
    departures: &Vec<SerializedTimestamp>,
    graph: &TDGraph,
    choice_algorithm: ChoiceAlgorithm,
    max_alternatives: u32,
    write_sumo_alternatives: bool,
    seed: i32,
    edge_indices_to_id: &Vec<String>,
    keep_route_probability: f64,
) {
    let keep_routes: Vec<bool> = if keep_route_probability <= 0.0 {
        vec![false; shortest_paths.len()]
    } else if keep_route_probability >= 1.0 {
        vec![true; shortest_paths.len()]
    } else {
        let mut rng: rngs::StdRng = SeedableRng::seed_from_u64(seed.abs() as u64);
        shortest_paths.iter().map(|_| rng.random_bool(keep_route_probability)).collect()
    };

    postprocess(
        input_dir,
        input_prefix,
        iteration,
        shortest_paths,
        travel_times,
        departures,
        graph,
        choice_algorithm,
        max_alternatives,
        write_sumo_alternatives,
        seed,
        edge_indices_to_id,
        &keep_routes,
        false,
    );
}

fn postprocess(
    input_dir: &Path,
    input_prefix: &String,
    iteration: u32,
    shortest_paths: &Vec<Vec<EdgeId>>,
    travel_times: &Vec<FlWeight>,
    departures: &Vec<SerializedTimestamp>,
    graph: &TDGraph,
    choice_algorithm: ChoiceAlgorithm,
    max_alternatives: u32,
    write_sumo_alternatives: bool,
    seed: i32,
    edge_indices_to_id: &Vec<String>,
    keep_routes: &Vec<bool>,
    skip_relative_gap: bool,
) {
    let current_iteration_dir = input_dir.join(format!("{:0>3}", iteration));

    // init all_routes with the previous alternatives
    let alternative_paths = if iteration > 0 {
        // load previous alternatives from input_dir
        let previous_iteration_dir = input_dir.join(format!("{:0>3}", iteration - 1));
        let old_alternative_paths: AlternativePathsForDTA = AlternativePathsForDTA::reconstruct(&previous_iteration_dir.join(DIR_DTA));

        if !skip_relative_gap {
            // get choices from old_alternative_paths to calculate relative gap
            set_relative_gap_with_previous_paths(&old_alternative_paths.get_chosen_paths(), graph, &input_dir, travel_times, departures);
        }

        // merge previous alternatives with current shortest paths
        let mut new_alternative_paths = old_alternative_paths.update_alternatives_with_new_paths(&shortest_paths, &travel_times, &departures, &graph);

        new_alternative_paths.perform_choice_model(&old_alternative_paths, &choice_algorithm, max_alternatives, &keep_routes, seed);

        new_alternative_paths
    } else {
        if !skip_relative_gap {
            // initialize relative gap file with 0.0 for the first iteration
            append_relative_gap_to_file(0.0, &input_dir);
        }
        AlternativePathsForDTA::init(shortest_paths, travel_times)
    };

    let (path_sets, costs, probabilities, choices) = transform_alternative_paths_for_dta_to_vectors(&alternative_paths);

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
        write_sumo_alternatives,
    );

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

pub fn set_relative_gap_with_previous_paths(
    previous_paths: &Vec<&Vec<u32>>,
    graph: &TDGraph,
    input_dir: &Path,
    travel_times: &Vec<FlWeight>,
    departures: &Vec<SerializedTimestamp>,
) {
    let simulated_tts: Vec<f64> = previous_paths
        .iter()
        .enumerate()
        .map(|(i, path)| f64::from(graph.get_travel_time_along_path(Timestamp::from_millis(departures[i]), path)))
        .collect();

    let rel_gap = get_relative_gap(&travel_times.iter().map(|tt| f64::from(*tt)).collect(), &simulated_tts);

    append_relative_gap_to_file(rel_gap, &input_dir);
}
