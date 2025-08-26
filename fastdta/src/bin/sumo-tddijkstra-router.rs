use std::path::Path;

use conversion::FILE_EDGE_INDICES_TO_ID;
use conversion::sumo::sumo_to_new_graph_weights::get_graph_with_travel_times_from_previous_iteration;
use fastdta::alternative_path_assembler::assemble_alternative_paths;
use fastdta::choice::{self};
use fastdta::cli;
use fastdta::cli::Parser;
use fastdta::query::get_paths_with_dijkstra_queries;
use rust_road_router::algo::dijkstra::query::floating_td_dijkstra::Server;
use rust_road_router::io::read_strings_from_file;
use rust_road_router::report::measure;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::RouterArgs::parse();

    let input_dir = Path::new(&args.input_dir);
    let input_prefix = args.input_prefix;
    let iteration = args.iteration;

    log(&input_dir.display().to_string(), iteration, "startup", 0);

    let choice_algorithm = match args.route_choice_method.as_str() {
        choice::LOGIT => choice::ChoiceAlgorithm::create_logit(args.logit_beta, args.logit_gamma, args.logit_theta),
        choice::GAWRON => choice::ChoiceAlgorithm::create_gawron(args.gawron_a, args.gawron_beta),
        _ => panic!("Unknown choice algorithm: {}", args.route_choice_method),
    };

    assert!(args.max_alternatives > 0, "max_alternatives must be greater than 0");

    let ((edge_ids, graph), duration) = measure(|| {
        let edge_ids: Vec<String> = read_strings_from_file(&input_dir.join(FILE_EDGE_INDICES_TO_ID)).unwrap();
        let graph = get_graph_with_travel_times_from_previous_iteration(input_dir, iteration, &edge_ids);

        (edge_ids, graph)
    });
    log(&input_dir.display().to_string(), iteration, "preprocessing", duration.as_nanos());

    let ((shortest_paths, travel_times, departures), duration) =
        measure(|| get_paths_with_dijkstra_queries(&mut Server::new(graph.clone()), input_dir, &graph));
    log(&input_dir.display().to_string(), iteration, "dijkstra routing", duration.as_nanos());

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

    log(&input_dir.display().to_string(), iteration, "assembling alternative paths", duration.as_nanos());

    Ok(())
}

/// Logs the operation with the duration in nanoseconds within a certain iteration of certain run identified by identifier.
/// The format is: "sumo-tdcch-router; <identifier>; <iteration>; <operation>; <duration_in_nanos>"
fn log(identifier: &str, iteration: u32, operation: &str, duration_in_nanos: u128) {
    println!("sumo-tddijkstra-router; {}; {}; {}; {}", identifier, iteration, operation, duration_in_nanos);
}
