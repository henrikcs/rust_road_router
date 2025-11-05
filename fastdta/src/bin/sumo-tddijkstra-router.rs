use std::path::Path;

use fastdta::alternative_path_assembler::assemble_alternative_paths;
use fastdta::cli;
use fastdta::cli::Parser;
use fastdta::logger::Logger;
use fastdta::query::get_paths_with_dijkstra;
use fastdta::route::get_graph_data_for_dijkstra;
use rust_road_router::report::measure;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::RouterArgs::parse();

    let input_dir = Path::new(&args.input_dir);
    let input_prefix = &args.input_prefix;
    let iteration = args.iteration;

    let choice_algorithm = args.get_choice_algorithm();

    assert!(args.max_alternatives > 0, "max_alternatives must be greater than 0");

    let logger = Logger::new("sumo-tddijkstra-router", &input_dir.display().to_string(), iteration as i32);

    let ((edge_ids, graph), duration) = measure(|| get_graph_data_for_dijkstra(input_dir, iteration));
    logger.log("preprocessing", duration.as_nanos());

    let ((shortest_paths, travel_times, departures), duration) = measure(|| get_paths_with_dijkstra(input_dir, &graph));
    logger.log("dijkstra routing", duration.as_nanos());

    let write_sumo_alternatives =
        args.no_write_sumo_alternatives == "false" || args.no_write_sumo_alternatives == "0" || args.no_write_sumo_alternatives == "False";

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
            args.max_alternatives,
            write_sumo_alternatives,
            args.seed.unwrap_or(rand::random::<i32>()),
            &edge_ids,
        )
    });

    logger.log("assembling alternative paths", duration.as_nanos());

    Ok(())
}
