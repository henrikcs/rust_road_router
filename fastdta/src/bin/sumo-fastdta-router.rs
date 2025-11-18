use std::path::Path;

use conversion::{FILE_EDGE_DEFAULT_TRAVEL_TIMES, SerializedTravelTime};
use fastdta::calibrate_traffic_model::calibrate_traffic_model;
use fastdta::cli;
use fastdta::cli::Parser;
use fastdta::customize::customize;
use fastdta::logger::Logger;
use fastdta::postprocess::{prepare_next_iteration, set_relative_gap_with_previous_alternative_paths};
use fastdta::query::{get_paths_with_cch, get_paths_with_dijkstra};
use fastdta::relative_gap::append_relative_gap_to_file;
use fastdta::route::{get_graph_data_for_cch, get_graph_data_for_dijkstra, get_graph_data_for_fast_dta, get_paths_by_samples};
use fastdta::sampler::sample;
use rust_road_router::io::Load;
use rust_road_router::report::measure;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::FastDtaArgs::parse();

    let input_dir = Path::new(&args.router_args.input_dir);
    let input_prefix = &args.router_args.input_prefix;
    let iteration = args.router_args.iteration;

    let choice_algorithm = args.router_args.get_choice_algorithm();
    let traffic_model_type = args.get_traffic_model();
    let samples = args.get_samples();

    assert!(args.router_args.max_alternatives > 0, "max_alternatives must be greater than 0");

    let logger = Logger::new("sumo-fastdta-router", &input_dir.display().to_string(), iteration as i32);

    let ((edge_ids, query_data, mut meandata, alternative_paths_for_dta, default_tts_sec), duration) = measure(|| {
        let (e, q, m, a) = get_graph_data_for_fast_dta(input_dir, iteration);

        let t = Vec::<SerializedTravelTime>::load_from(&input_dir.join(FILE_EDGE_DEFAULT_TRAVEL_TIMES)).unwrap();

        (e, q, m, a, t.iter().map(|stt| *stt as f64 / 1000.0).collect::<Vec<f64>>())
    });
    logger.log("preprocessing", duration.as_nanos());

    let (traffic_model, duration) = measure(|| {
        // for each edge, go into each interval and extract lane_density and speed as data points
        // then use these data points to calibrate the traffic model parameters of ModifiedLee
        calibrate_traffic_model(&meandata, &edge_ids, &default_tts_sec, &traffic_model_type)
    });

    logger.log("calibration", duration.as_nanos());

    let (samples, duration) = measure(|| sample(&samples, query_data.0.len(), args.router_args.seed.unwrap_or(rand::random::<i32>())));
    logger.log("sample", duration.as_nanos());

    let ((graph, paths, travel_times, departures), duration) = measure(|| {
        get_paths_by_samples(
            &input_dir,
            iteration,
            &logger,
            &query_data,
            &samples,
            &traffic_model,
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
            let (edge_ids, graph, cch) = get_graph_data_for_cch(input_dir, iteration);
            let customized_graph = customize(&cch, &graph);

            let (shortest_paths, shortest_travel_times, departures) = get_paths_with_cch(&cch, &customized_graph, input_dir, &graph);

            set_relative_gap_with_previous_alternative_paths(&alternative_paths_for_dta, &graph, &input_dir, &shortest_travel_times, &departures);
        }
    });

    logger.log("postprocessing", duration.as_nanos());

    Ok(())
}
