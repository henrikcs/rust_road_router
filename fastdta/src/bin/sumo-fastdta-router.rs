use std::path::Path;

use fastdta::cli;
use fastdta::cli::Parser;
use fastdta::logger::Logger;
use fastdta::postprocess::{prepare_next_iteration, set_relative_gap_with_previous_alternative_paths};
use fastdta::query::get_paths_with_dijkstra;
use fastdta::relative_gap::append_relative_gap_to_file;
use fastdta::route::{get_graph_data_for_dijkstra, get_graph_data_for_fast_dta, get_paths_by_samples};
use fastdta::sampler::sample;
use rust_road_router::report::measure;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::FastDtaArgs::parse();

    let input_dir = Path::new(&args.router_args.input_dir);
    let input_prefix = &args.router_args.input_prefix;
    let iteration = args.router_args.iteration;

    let choice_algorithm = args.router_args.get_choice_algorithm();
    let vdf = args.get_vdf();
    let samples = args.get_samples();

    assert!(args.router_args.max_alternatives > 0, "max_alternatives must be greater than 0");

    let logger = Logger::new("sumo-fastdta-router", &input_dir.display().to_string(), iteration as i32);

    let ((edge_ids, query_data, mut meandata, alternative_paths_for_dta), duration) = measure(|| get_graph_data_for_fast_dta(input_dir, iteration));
    logger.log("preprocessing", duration.as_nanos());

    let (samples, duration) = measure(|| sample(&samples, query_data.0.len(), args.router_args.seed.unwrap_or(rand::random::<i32>())));
    logger.log("sample", duration.as_nanos());

    let ((graph, paths, travel_times, departures), duration) = measure(|| {
        get_paths_by_samples(
            &input_dir,
            iteration,
            &logger,
            &query_data,
            &samples,
            vdf,
            &alternative_paths_for_dta,
            &mut meandata,
            &edge_ids,
        )
    });

    logger.log("fastdta routing", duration.as_nanos());

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
            // initialize relative gap file with 0.0 for the first iteration
            append_relative_gap_to_file(0.0, &input_dir);
        } else {
            // get graph from previous iteration
            let (edge_ids, graph) = get_graph_data_for_dijkstra(input_dir, iteration);

            let (shortest_paths, shortest_travel_times, departures) = get_paths_with_dijkstra(input_dir, &graph);

            set_relative_gap_with_previous_alternative_paths(&alternative_paths_for_dta, &graph, &input_dir, &shortest_travel_times, &departures);
        }
    });

    logger.log("postprocessing", duration.as_nanos());

    Ok(())
}
