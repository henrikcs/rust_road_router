use std::path::Path;

use conversion::{
    FILE_EDGE_DEFAULT_TRAVEL_TIMES, FILE_EDGE_LANES, FILE_EDGE_LENGTHS, SerializedTravelTime,
    sumo::sumo_to_new_graph_weights::extract_interpolation_points_from_meandata,
};
use fastdta::calibrate_traffic_model::calibrate_traffic_models;
use fastdta::cli;
use fastdta::cli::Parser;
use fastdta::customize::customize;
use fastdta::logger::Logger;
use fastdta::path_processor::adjust_weights_in_graph_by_following_paths;
use fastdta::postprocess::prepare_next_iteration_for_fastdta2;
use fastdta::preprocess::get_cch;
use fastdta::preprocess_routes::get_graph_data_for_fastdta2;
use fastdta::query::get_paths_with_cch_queries;
use rust_road_router::datastr::graph::floating_time_dependent::{FlWeight, TDGraph, Timestamp};
use rust_road_router::io::{Load, Reconstruct};
use rust_road_router::report::measure;

/// Debug function to check if paths from SP routing and FastDTA2 routing are identical
fn debug_check_path_equality(sp_paths: &[Vec<u32>], fastdta2_paths: &[Vec<u32>], iteration: u32) {
    assert_eq!(sp_paths.len(), fastdta2_paths.len(), "Path count mismatch");

    let mut identical_count = 0;
    let mut different_count = 0;

    for (i, (sp_path, fastdta2_path)) in sp_paths.iter().zip(fastdta2_paths.iter()).enumerate() {
        if sp_path == fastdta2_path {
            identical_count += 1;
        } else {
            different_count += 1;
            if different_count <= 5 {
                // Print first 5 differences
                eprintln!("[DEBUG] Iteration {}, Query {}: Paths differ", iteration, i);
                eprintln!("  SP path length: {}, FastDTA2 path length: {}", sp_path.len(), fastdta2_path.len());
            }
        }
    }

    eprintln!(
        "[DEBUG] Iteration {}: {} identical paths, {} different paths out of {} total",
        iteration,
        identical_count,
        different_count,
        sp_paths.len()
    );

    if identical_count == sp_paths.len() {
        eprintln!("[DEBUG] WARNING: All paths are identical! FastDTA2 routing added no new alternatives.");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::FastDtaArgs::parse();

    let input_dir = Path::new(&args.router_args.input_dir);
    let input_prefix = &args.router_args.input_prefix;
    let iteration = args.router_args.iteration;

    let choice_algorithm = args.router_args.get_choice_algorithm();
    let traffic_model_type = args.get_traffic_model();
    let keep_route_probability = args.router_args.keep_route_probability.unwrap_or(0.0);

    assert!(args.router_args.max_alternatives > 0, "max_alternatives must be greater than 0");

    let logger = Logger::new("sumo-fastdta2-router", &input_dir.display().to_string(), iteration as i32);

    // Load graph data including meandata, traffic models, and alternative paths from previous iteration
    let ((edge_ids, query_data, mut meandata, previous_alternative_paths, mut traffic_model_data, keep_routes), duration) =
        measure(|| get_graph_data_for_fastdta2(input_dir, iteration, traffic_model_type, keep_route_probability));

    logger.log("preprocessing", duration.as_nanos());

    // Calibrate traffic models using meandata from the previous simulation
    let (_, duration) = measure(|| {
        calibrate_traffic_models(&mut traffic_model_data, &mut meandata, &edge_ids, args.calibration_data_points_threshold);
    });

    logger.log("calibration", duration.as_nanos());

    // Load additional data needed for routing
    let free_flow_tts_ms = &Vec::<SerializedTravelTime>::load_from(&input_dir.join(FILE_EDGE_DEFAULT_TRAVEL_TIMES)).unwrap();
    let free_flow_tts: Vec<f64> = free_flow_tts_ms.iter().map(|&tt| tt as f64 / 1000.0).collect();
    let edge_lengths = Vec::<f64>::load_from(&input_dir.join(FILE_EDGE_LENGTHS)).unwrap();
    let edge_lanes = Vec::<u32>::load_from(&input_dir.join(FILE_EDGE_LANES)).unwrap();

    // STEP 1: Compute shortest paths SP on network N with simulated weights w_i using CCH
    let ((mut graph, cch, departures, _sp_paths, _sp_travel_times, alternative_paths), duration) = measure(|| {
        // Reconstruct the graph with travel times from the previous iteration
        let mut graph: TDGraph = TDGraph::reconstruct_from(&input_dir).expect("Failed to reconstruct the time-dependent graph");

        // If iteration > 0, apply meandata to graph weights
        if iteration > 0 {
            let (first_ipp_of_arc, ipp_travel_time, ipp_departure_time) = extract_interpolation_points_from_meandata(&meandata, &edge_ids, &free_flow_tts_ms);
            graph = TDGraph::new(
                Vec::from(graph.first_out()),
                Vec::from(graph.head()),
                first_ipp_of_arc,
                ipp_departure_time,
                ipp_travel_time,
            );
        }

        let cch = get_cch(input_dir, &graph);
        let customized_graph = customize(&cch, &graph);

        // Get shortest paths using CCH
        let (sp_paths, sp_travel_times, _departures) = get_paths_with_cch_queries(
            &cch,
            &customized_graph,
            &query_data.0,
            &query_data.1,
            &query_data.2,
            &query_data.3,
            &query_data.4,
            &graph,
        );

        // STEP 2: Add SP to alternative paths and perform choice model
        let mut alternative_paths = previous_alternative_paths.update_alternatives_with_new_paths(
            &sp_paths,
            &sp_travel_times,
            &query_data.2, // departures as SerializedTimestamp
            &graph,
        );

        // Perform choice model to get preferred paths P
        alternative_paths.perform_choice_model(
            &previous_alternative_paths,
            &choice_algorithm,
            args.router_args.max_alternatives,
            &keep_routes,
            args.router_args.seed.unwrap_or(rand::random::<i32>()),
        );

        (graph, cch, query_data.2.clone(), sp_paths, sp_travel_times, alternative_paths)
    });

    logger.log("first routing", duration.as_nanos());

    // STEP 3: Get preferred paths P from choice model
    let preferred_paths = alternative_paths.get_chosen_paths();
    let previous_paths = previous_alternative_paths.get_chosen_paths();

    // STEP 4: Temporarily calculate weights w_i' by following paths P on N using traffic model
    let ((fastdta2_paths, fastdta2_travel_times_on_original), duration) = measure(|| {
        // Clone meandata for temporary weight calculation
        let mut temp_meandata = meandata.clone();

        // Convert preferred paths to owned vectors for the API
        let preferred_paths_owned: Vec<Vec<u32>> = preferred_paths.iter().map(|p| (*p).clone()).collect();
        let departures_timestamps: Vec<Timestamp> = departures.iter().map(|&d| Timestamp::from_millis(d)).collect();

        // Adjust weights in graph by following paths P (subtracting old paths, adding new paths)
        adjust_weights_in_graph_by_following_paths(
            &mut graph,
            &previous_paths,
            &preferred_paths_owned,
            &departures_timestamps,
            &mut temp_meandata.intervals,
            &edge_ids,
            &edge_lengths,
            &free_flow_tts,
            &traffic_model_data.traffic_models,
            &edge_lanes,
        );

        // STEP 5: Compute shortest paths P' for all vehicles on N with weights w_i'
        let customized_graph_prime = customize(&cch, &graph);

        let (fastdta2_paths, _fastdta2_travel_times, _) = get_paths_with_cch_queries(
            &cch,
            &customized_graph_prime,
            &query_data.0,
            &query_data.1,
            &query_data.2,
            &query_data.3,
            &query_data.4,
            &graph,
        );

        // STEP 6: Compute travel times of P' on N with w_i (original weights)
        // Reconstruct original graph to get correct travel times
        let mut original_graph: TDGraph = TDGraph::reconstruct_from(&input_dir).expect("Failed to reconstruct the time-dependent graph");
        if iteration > 0 {
            let (first_ipp_of_arc, ipp_travel_time, ipp_departure_time) = extract_interpolation_points_from_meandata(&meandata, &edge_ids, &free_flow_tts_ms);
            original_graph = TDGraph::new(
                Vec::from(original_graph.first_out()),
                Vec::from(original_graph.head()),
                first_ipp_of_arc,
                ipp_departure_time,
                ipp_travel_time,
            );
        }

        // Calculate travel times of FastDTA2 paths on original graph
        let fastdta2_travel_times_on_original: Vec<FlWeight> = fastdta2_paths
            .iter()
            .enumerate()
            .map(|(i, path)| original_graph.get_travel_time_along_path(Timestamp::from_millis(departures[i]), path))
            .collect();

        (fastdta2_paths, fastdta2_travel_times_on_original)
    });

    logger.log("second routing", duration.as_nanos());

    // STEP 7: Add P' to alternative paths
    let (alternative_paths, duration) = measure(|| {
        // Reconstruct original graph for the final update
        let mut original_graph: TDGraph = TDGraph::reconstruct_from(&input_dir).expect("Failed to reconstruct the time-dependent graph");
        if iteration > 0 {
            let (first_ipp_of_arc, ipp_travel_time, ipp_departure_time) = extract_interpolation_points_from_meandata(&meandata, &edge_ids, &free_flow_tts_ms);
            original_graph = TDGraph::new(
                Vec::from(original_graph.first_out()),
                Vec::from(original_graph.head()),
                first_ipp_of_arc,
                ipp_departure_time,
                ipp_travel_time,
            );
        }

        // Debug: Check if SP paths and FastDTA2 paths are identical
        debug_check_path_equality(&_sp_paths, &fastdta2_paths, iteration);

        // Add FastDTA2 paths to alternatives (which already contains SP paths from STEP 2)
        let state_before_fastdta2 = alternative_paths.clone();
        let mut alternative_paths =
            alternative_paths.update_alternatives_with_new_paths(&fastdta2_paths, &fastdta2_travel_times_on_original, &departures, &original_graph);

        // STEP 8: Perform choice model again with updated alternatives
        // For the second choice model, we use the state before adding FastDTA2 as "previous"
        alternative_paths.perform_choice_model(
            &state_before_fastdta2,
            &choice_algorithm,
            args.router_args.max_alternatives,
            &keep_routes,
            args.router_args.seed.unwrap_or(rand::random::<i32>()) + 1, // Use different seed for second choice
        );

        alternative_paths
    });

    logger.log("second choice model", duration.as_nanos());

    // STEP 9: Prepare output for next iteration
    let (_, duration) = measure(|| {
        // Reconstruct original graph for postprocessing
        let mut original_graph: TDGraph = TDGraph::reconstruct_from(&input_dir).expect("Failed to reconstruct the time-dependent graph");
        if iteration > 0 {
            let (first_ipp_of_arc, ipp_travel_time, ipp_departure_time) = extract_interpolation_points_from_meandata(&meandata, &edge_ids, &free_flow_tts_ms);
            original_graph = TDGraph::new(
                Vec::from(original_graph.first_out()),
                Vec::from(original_graph.head()),
                first_ipp_of_arc,
                ipp_departure_time,
                ipp_travel_time,
            );
        }

        prepare_next_iteration_for_fastdta2(
            input_dir,
            input_prefix,
            iteration,
            &alternative_paths,
            &departures,
            &original_graph,
            args.router_args.get_write_sumo_alternatives(),
            &edge_ids,
        );

        traffic_model_data.deconstruct(&input_dir).unwrap();
    });

    logger.log("postprocessing", duration.as_nanos());

    Ok(())
}
