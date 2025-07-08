use std::path::Path;

use conversion::FILE_EDGE_INDICES_TO_ID;
use conversion::sumo::paths_to_sumo_routes_converter::write_paths_as_sumo_routes;
use conversion::sumo::sumo_to_new_graph_weights::get_graph_with_travel_times_from_previous_iteration;
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

    let edge_ids: Vec<String> = read_strings_from_file(&input_dir.join(FILE_EDGE_INDICES_TO_ID)).unwrap();

    let graph = get_graph_with_travel_times_from_previous_iteration(input_dir, iteration, &edge_ids);
    let cch = get_cch(input_dir, &graph);

    let customized_graph = customize(&cch, &graph);

    let (paths, travel_times, departures) = get_paths_from_queries(&cch, &customized_graph, &input_dir);

    write_paths_as_sumo_routes(&input_dir, &input_prefix, iteration, &paths, &departures, &edge_ids);

    Ok(())
}
