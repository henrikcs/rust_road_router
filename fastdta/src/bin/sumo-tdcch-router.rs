use std::path::Path;

use conversion::FILE_EDGE_INDICES_TO_ID;
use conversion::sumo::sumo_to_new_graph_weights::get_graph_with_travel_times_from_previous_iteration;
use fastdta::alternative_path_assembler::assemble_alternative_paths;
use fastdta::choice::{self};
use fastdta::cli;
use fastdta::cli::Parser;
use fastdta::customize::customize;
use fastdta::preprocess::get_cch;
use fastdta::query::get_paths_from_queries;
use rust_road_router::io::read_strings_from_file;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let _reporter = enable_reporting("tdcch_customization");
    // report!("num_threads", rayon::current_num_threads());

    let args = cli::RouterArgs::parse();

    let input_dir = Path::new(&args.input_dir);
    let input_prefix = args.input_prefix;
    let iteration = args.iteration;

    dbg!(&input_prefix);
    dbg!(&input_dir);

    let choice_algorithm = match args.route_choice_method.as_str() {
        choice::LOGIT => choice::ChoiceAlgorithm::create_logit(args.logit_theta),
        choice::GAWRON => choice::ChoiceAlgorithm::create_gawron(args.gawron_a, args.gawron_beta),
        _ => panic!("Unknown choice algorithm: {}", args.route_choice_method),
    };

    assert!(args.max_alternatives > 0, "max_alternatives must be greater than 0");

    let edge_ids: Vec<String> = read_strings_from_file(&input_dir.join(FILE_EDGE_INDICES_TO_ID)).unwrap();

    let graph = get_graph_with_travel_times_from_previous_iteration(input_dir, iteration, &edge_ids);
    let cch = get_cch(input_dir, &graph);

    let customized_graph = customize(&cch, &graph);

    let (shortest_paths, travel_times, departures) = get_paths_from_queries(&cch, &customized_graph, &input_dir);

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
        args.seed.unwrap_or(rand::random::<i32>()),
        &edge_ids,
    );

    Ok(())
}
