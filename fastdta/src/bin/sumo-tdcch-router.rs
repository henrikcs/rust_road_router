use fastdta::cli;
use fastdta::cli::Parser;
use fastdta::customize::customize;
use fastdta::logger::Logger;
use fastdta::postprocess::prepare_next_iteration;
use fastdta::preprocess_routes::get_graph_data_for_cch;
use fastdta::query::get_paths_with_cch;
use rust_road_router::report::measure;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::RouterArgs::parse();

    let input_dir = Path::new(&args.input_dir);
    let input_prefix = &args.input_prefix;
    let iteration = args.iteration;
    let keep_route_probability = args.keep_route_probability.unwrap_or(0.0);

    let logger = Logger::new("sumo-tdcch-router", &input_dir.display().to_string(), iteration as i32);

    let choice_algorithm = args.get_choice_algorithm();

    assert!(args.max_alternatives > 0, "max_alternatives must be greater than 0");

    let ((edge_ids, graph, cch), duration) = measure(|| get_graph_data_for_cch(input_dir, iteration));
    logger.log("preprocessing", duration.as_nanos());

    let (customized_graph, duration) = measure(|| customize(&cch, &graph));
    logger.log("cch customization", duration.as_nanos());

    let ((shortest_paths, travel_times, departures), duration) = measure(|| get_paths_with_cch(&cch, &customized_graph, &input_dir, &graph));
    logger.log("cch routing", duration.as_nanos());

    for shortest_path in &shortest_paths {
        if shortest_path.is_empty() {
            println!("Found an empty path!");
        }
    }

    let write_sumo_alternatives =
        args.no_write_sumo_alternatives == "false" || args.no_write_sumo_alternatives == "0" || args.no_write_sumo_alternatives == "False";

    let (_, duration) = measure(|| {
        prepare_next_iteration(
            &input_dir,
            &input_prefix,
            iteration,
            &shortest_paths,
            &travel_times,
            &departures,
            &graph,
            choice_algorithm,
            args.max_alternatives,
            write_sumo_alternatives,
            args.seed.unwrap_or(rand::random::<i32>()),
            &edge_ids,
            keep_route_probability,
        )
    });

    logger.log("postprocessing", duration.as_nanos());

    Ok(())
}
