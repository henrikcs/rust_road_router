use std::path::Path;

use fastdta::alternative_path_assembler::assemble_alternative_paths;
use fastdta::cli;
use fastdta::cli::Parser;
use fastdta::logger::Logger;
use fastdta::route::{get_graph_data_for_fast_dta, get_paths_by_samples};
use fastdta::sampler::sample;
use rust_road_router::report::measure;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::FastDtaArgs::parse();

    let input_dir = Path::new(&args.router_args.input_dir);
    let input_prefix = &args.router_args.input_prefix;
    let iteration = args.router_args.iteration;

    let choice_algorithm = args.router_args.get_choice_algorithm();
    let vdf = args.get_vdf();

    assert!(args.router_args.max_alternatives > 0, "max_alternatives must be greater than 0");

    let logger = Logger::new("sumo-fastdta-router", &input_dir.display().to_string(), iteration as i32);

    let ((edge_ids, query_data, mut meandata, old_paths), duration) = measure(|| get_graph_data_for_fast_dta(input_dir, iteration));
    logger.log("preprocessing", duration.as_nanos());

    let (samples, duration) = measure(|| {
        sample(
            &args.samples.unwrap_or(vec![0.1, 0.2, 0.3, 0.4]),
            query_data.0.len(),
            args.router_args.seed.unwrap_or(rand::random::<i32>()),
        )
    });
    logger.log("sample", duration.as_nanos());

    let ((graph, shortest_paths, travel_times, departures), duration) =
        measure(|| get_paths_by_samples(&input_dir, &logger, &query_data, &samples, vdf, &old_paths, &mut meandata, &edge_ids));

    logger.log("fastdta routing", duration.as_nanos());

    let (_, duration) = measure(|| {
        assemble_alternative_paths(
            &input_dir,
            &input_prefix,
            iteration,
            &shortest_paths,
            &travel_times,
            &departures,
            &graph,
            choice_algorithm,
            args.router_args.max_alternatives,
            args.router_args.get_write_some_alternatives(),
            args.router_args.seed.unwrap_or(rand::random::<i32>()),
            &edge_ids,
        )
    });

    logger.log("assembling alternative paths", duration.as_nanos());

    Ok(())
}
