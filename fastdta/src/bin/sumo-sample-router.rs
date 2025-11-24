use std::path::Path;

use fastdta::cli;
use fastdta::cli::Parser;
use fastdta::customize::customize;
use fastdta::logger::Logger;
use fastdta::postprocess::prepare_next_iteration;
use fastdta::postprocess::set_relative_gap_with_previous_paths;
use fastdta::query::get_paths_with_cch;
use fastdta::relative_gap::append_relative_gap_to_file;
use fastdta::route::get_graph_data_for_cch;
use fastdta::sampler::sample;
use fastdta::sumo_sample_routing::get_paths_by_samples_with_sumo;
use rust_road_router::io::read_strings_from_file;
use rust_road_router::report::measure;

use conversion::FILE_EDGE_INDICES_TO_ID;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::SumoSampleRouterArgs::parse();
    let router_args = &args.router_args;
    let input_dir = Path::new(&router_args.input_dir);
    let input_prefix = &router_args.input_prefix;
    let iteration = router_args.iteration;
    let net_file_string = router_args.net_file.as_ref().unwrap();
    let net_file = Path::new(&net_file_string);
    let aggregation = args.aggregation;

    let choice_algorithm = router_args.get_choice_algorithm();
    let samples = args.get_samples();

    assert!(args.router_args.max_alternatives > 0, "max_alternatives must be greater than 0");

    let logger = Logger::new("sumo-sample-router", &input_dir.display().to_string(), iteration as i32);

    // Read edge IDs and query data
    let (edge_ids, duration) = measure(|| read_strings_from_file(&input_dir.join(FILE_EDGE_INDICES_TO_ID)).expect("Failed to read edge IDs"));
    logger.log("read edge ids", duration.as_nanos());

    let (query_data, duration) = measure(|| fastdta::query::read_queries(input_dir));
    logger.log("read queries", duration.as_nanos());

    // Generate samples
    let (samples, duration) = measure(|| sample(&samples, query_data.0.len(), args.router_args.seed.unwrap_or(rand::random::<i32>())));
    logger.log("sample", duration.as_nanos());

    // Get previous paths if iteration > 0
    let alternative_paths_holder = if iteration > 0 {
        let previous_iteration_dir = input_dir.join(format!("{:0>3}", iteration - 1));
        Some(fastdta::alternative_paths::AlternativePathsForDTA::reconstruct(
            &previous_iteration_dir.join(conversion::DIR_DTA),
        ))
    } else {
        None
    };
    let previous_paths = alternative_paths_holder.as_ref().map(|ap| ap.get_chosen_paths()).unwrap_or_default();

    // Route all queries using samples with SUMO simulation
    let ((graph, paths, travel_times, departures), duration) =
        measure(|| get_paths_by_samples_with_sumo(&input_dir, &net_file, iteration, aggregation, &logger, &query_data, &samples, &edge_ids));

    logger.log("sumo-based routing", duration.as_nanos());

    // Prepare next iteration
    let (_, duration) = measure(|| {
        prepare_next_iteration(
            &input_dir,
            &input_prefix,
            iteration,
            &paths,
            &travel_times,
            &departures,
            &graph,
            choice_algorithm,
            args.router_args.max_alternatives,
            args.router_args.get_write_sumo_alternatives(),
            args.router_args.seed.unwrap_or(rand::random::<i32>()),
            &edge_ids,
            true,
        );

        if iteration == 0 {
            // Initialize relative gap file with 0.0 for the first iteration
            append_relative_gap_to_file(0.0, &input_dir);
        } else {
            // Get graph from previous iteration and calculate relative gap
            let (_, graph, cch) = get_graph_data_for_cch(input_dir, iteration);
            let customized_graph = customize(&cch, &graph);

            let (_, shortest_travel_times, departures) = get_paths_with_cch(&cch, &customized_graph, input_dir, &graph);

            set_relative_gap_with_previous_paths(&previous_paths, &graph, &input_dir, &shortest_travel_times, &departures);
        }
    });

    logger.log("postprocessing", duration.as_nanos());

    Ok(())
}
