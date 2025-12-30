use std::path::Path;

use fastdta::calibrate_traffic_model::calibrate_traffic_models;
use fastdta::cli;
use fastdta::cli::Parser;
use fastdta::customize::customize;
use fastdta::logger::Logger;
use fastdta::path_processor::adjust_weights_in_graph_by_following_paths;
use fastdta::postprocess::prepare_next_iteration_for_fastdta2;
use fastdta::preprocess_routes::{get_graph_data_for_cch, get_graph_data_for_fastdta2};
use fastdta::query::get_paths_with_cch_queries;
use rust_road_router::datastr::graph::floating_time_dependent::{FlWeight, Timestamp};
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

    let seed = args.router_args.seed.unwrap_or(42);

    assert!(args.router_args.max_alternatives > 0, "max_alternatives must be greater than 0");

    let logger = Logger::new("sumo-fastdta2-router", &input_dir.display().to_string(), iteration as i32);

    // Load graph data including meandata, traffic models, and alternative paths from previous iteration
    // Also load graph and CCH for routing
    let (
        (
            edge_ids,
            query_data,
            mut meandata,
            previous_alternative_paths,
            mut traffic_model_data,
            keep_routes,
            mut graph,
            original_graph_copy,
            cch,
            free_flow_tts,
            edge_lengths,
            edge_lanes,
        ),
        duration,
    ) = measure(|| {
        let (edge_ids, free_flow_tts, edge_lengths, edge_lanes, query_data, meandata, previous_alternative_paths, traffic_model_data, keep_routes) =
            get_graph_data_for_fastdta2(input_dir, iteration, traffic_model_type, keep_route_probability);

        // Reconstruct the graph with travel times from the previous iteration
        let (_, graph, cch) = get_graph_data_for_cch(input_dir, iteration);

        // Create a copy of the graph for later use
        let original_graph_copy = graph.clone();

        (
            edge_ids,
            query_data,
            meandata,
            previous_alternative_paths,
            traffic_model_data,
            keep_routes,
            graph,
            original_graph_copy,
            cch,
            free_flow_tts,
            edge_lengths,
            edge_lanes,
        )
    });

    logger.log("preprocessing", duration.as_nanos());

    // Calibrate traffic models using meandata from the previous simulation
    let (_, duration) = measure(|| {
        calibrate_traffic_models(&mut traffic_model_data, &mut meandata, &edge_ids, args.calibration_data_points_threshold);
    });

    logger.log("calibration", duration.as_nanos());

    // STEP 1: Customize graph for routing and Compute shortest paths SP on network N with simulated weights w_i using CCH
    let (customized_graph, duration) = measure(|| customize(&cch, &graph));

    logger.log("first customization", duration.as_nanos());

    let ((sp_paths, sp_travel_times, _), duration) = measure(|| {
        get_paths_with_cch_queries(
            &cch,
            &customized_graph,
            &query_data.0,
            &query_data.1,
            &query_data.2,
            &query_data.3,
            &query_data.4,
            &graph,
        )
    });

    logger.log("first routing", duration.as_nanos());

    // STEP 2: Add SP to alternative paths and perform choice model
    let (alternative_paths, duration) = measure(|| {
        let departures = &query_data.2;
        let mut alternative_paths = previous_alternative_paths.update_alternatives_with_new_paths(&sp_paths, &sp_travel_times, departures, &graph);

        // Perform choice model to get preferred paths P
        // choice model will modify costs inside alternative_paths
        alternative_paths.perform_choice_model(
            &previous_alternative_paths,
            &choice_algorithm,
            args.router_args.max_alternatives,
            &keep_routes,
            seed,
        );

        alternative_paths
    });

    logger.log("first choice model", duration.as_nanos());

    // STEP 3: Get preferred paths P from choice model
    let ((preferred_paths, previous_paths), duration) = measure(|| {
        let preferred_paths: Vec<Vec<u32>> = alternative_paths.get_chosen_paths().iter().map(|p| (*p).clone()).collect();
        let previous_paths = previous_alternative_paths.get_chosen_paths();
        (preferred_paths, previous_paths)
    });

    logger.log("get preferred paths", duration.as_nanos());

    // STEP 4: Temporarily calculate weights w_i' by following paths P on N using traffic model
    let (_, duration) = measure(|| {
        // Clone meandata for temporary weight calculation
        // let mut temp_meandata = meandata.clone();

        let departures: &Vec<u32> = &query_data.2;
        let departures_timestamps: Vec<Timestamp> = departures.iter().map(|&d| Timestamp::from_millis(d)).collect();

        // Adjust weights in graph by following paths P (subtracting old paths, adding new paths)
        adjust_weights_in_graph_by_following_paths(
            &mut graph,
            &previous_paths,
            &preferred_paths,
            &departures_timestamps,
            &mut meandata.intervals,
            &edge_ids,
            &edge_lengths,
            &free_flow_tts,
            &traffic_model_data.traffic_models,
            &edge_lanes,
        );
    });

    logger.log("adjust weights", duration.as_nanos());

    // STEP 5: Customize graph with adjusted weights
    let (customized_graph_prime, duration) = measure(|| customize(&cch, &graph));

    logger.log("second customization", duration.as_nanos());

    // STEP 5: Compute shortest paths P' for all vehicles on N with weights w_i'
    let (fastdta2_paths, duration) = measure(|| {
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
        fastdta2_paths
    });

    logger.log("second routing", duration.as_nanos());

    // STEP 6: Compute travel times of P' on N with w_i (original weights)
    let (fastdta2_travel_times_on_original, duration) = measure(|| {
        let departures: &Vec<u32> = &query_data.2;

        // Calculate travel times of FastDTA2 paths on original graph (use copy)
        let fastdta2_travel_times_on_original: Vec<FlWeight> = fastdta2_paths
            .iter()
            .enumerate()
            .map(|(i, path)| original_graph_copy.get_travel_time_along_path(Timestamp::from_millis(departures[i]), path))
            .collect();

        fastdta2_travel_times_on_original
    });

    logger.log("calculate travel times on original", duration.as_nanos());

    // STEP 7: Add P' to alternative paths
    let (mut alternative_paths, duration) = measure(|| {
        let departures: &Vec<u32> = &query_data.2;

        //only do this when built in debug mode
        #[cfg(debug_assertions)]
        {
            // Debug: Check if SP paths and FastDTA2 paths are identical
            debug_check_path_equality(&sp_paths, &fastdta2_paths, iteration);
        }

        // Add FastDTA2 paths to previous alternatives (which already contains SP paths from STEP 2)

        // add both the previously found shortest paths and the alternatives found by fastdta2
        let alternative_paths = previous_alternative_paths.update_alternatives_with_new_paths(
            &sp_paths,
            &sp_travel_times,
            departures, // departures as SerializedTimestamp
            &original_graph_copy,
        );

        alternative_paths.update_alternatives_with_new_paths(&fastdta2_paths, &fastdta2_travel_times_on_original, &departures, &original_graph_copy)
    });

    logger.log("add fastdta2 alternatives", duration.as_nanos());

    // STEP 8: Perform choice model again with updated alternatives
    let (_, duration) = measure(|| {
        // For the second choice model, we use the state before adding FastDTA2 as "previous"
        alternative_paths.perform_choice_model(
            &previous_alternative_paths,
            &choice_algorithm,
            args.router_args.max_alternatives,
            &keep_routes,
            seed,
        );
    });

    logger.log("second choice model", duration.as_nanos());

    // STEP 9: Prepare output for next iteration
    let (_, duration) = measure(|| {
        let departures: &Vec<u32> = &query_data.2;

        prepare_next_iteration_for_fastdta2(
            input_dir,
            input_prefix,
            iteration,
            &alternative_paths,
            &sp_travel_times,
            &departures,
            &original_graph_copy,
            args.router_args.get_write_sumo_alternatives(),
            &edge_ids,
        );

        traffic_model_data.deconstruct(&input_dir).unwrap();
    });

    logger.log("postprocessing", duration.as_nanos());

    Ok(())
}
