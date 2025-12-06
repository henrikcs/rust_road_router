use std::path::Path;

use conversion::FILE_QUERY_IDS;
use fastdta::calibrate_traffic_model::calibrate_traffic_models;
use fastdta::cli;
use fastdta::cli::Parser;
use fastdta::customize::customize;
use fastdta::logger::Logger;
use fastdta::postprocess::{prepare_next_iteration_for_sampled_routing, set_relative_gap_with_previous_paths};
use fastdta::preprocess_routes::{get_graph_data_for_cch, get_graph_data_for_fast_dta};
use fastdta::query::get_paths_with_cch;
use fastdta::relative_gap::{EPSILON_TRAVEL_TIME, append_relative_gap_to_file};
use fastdta::sampled_queries::{get_paths_by_samples, get_paths_by_samples_with_keep_routes};
use fastdta::sampler::sample;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;
use rust_road_router::datastr::graph::floating_time_dependent::{FlWeight, Timestamp};
use rust_road_router::io::read_strings_from_file;
use rust_road_router::report::measure;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::FastDtaArgs::parse();

    let input_dir = Path::new(&args.router_args.input_dir);
    let input_prefix = &args.router_args.input_prefix;
    let iteration = args.router_args.iteration;

    let choice_algorithm = args.router_args.get_choice_algorithm();
    let traffic_model_type = args.get_traffic_model();
    let samples = args.get_samples();
    let keep_route_probability = args.router_args.keep_route_probability.unwrap_or(0.0);

    assert!(args.router_args.max_alternatives > 0, "max_alternatives must be greater than 0");

    let logger = Logger::new("sumo-fastdta-router", &input_dir.display().to_string(), iteration as i32);

    let ((edge_ids, query_data, mut meandata, alternative_paths_from_dta, mut traffic_model_data, keep_routes), duration) =
        measure(|| get_graph_data_for_fast_dta(input_dir, iteration, traffic_model_type, keep_route_probability));

    logger.log("preprocessing", duration.as_nanos());

    let (_, duration) = measure(|| {
        calibrate_traffic_models(&mut traffic_model_data, &mut meandata, &edge_ids, args.calibration_data_points_threshold);
    });

    logger.log("calibration", duration.as_nanos());

    let (samples, duration) = measure(|| sample(&samples, query_data.0.len(), args.router_args.seed.unwrap_or(rand::random::<i32>())));
    logger.log("sample", duration.as_nanos());

    let previous_paths = alternative_paths_from_dta.get_chosen_paths();

    let ((graph, paths, travel_times, departures), duration) = measure(|| {
        if !args.keep_route_in_sampling {
            return get_paths_by_samples(
                &input_dir,
                iteration,
                &logger,
                &query_data,
                &samples,
                &traffic_model_data.traffic_models,
                &previous_paths,
                &mut meandata,
                &edge_ids,
            );
        }
        get_paths_by_samples_with_keep_routes(
            &input_dir,
            iteration,
            &logger,
            &query_data,
            &samples,
            &traffic_model_data.traffic_models,
            &previous_paths,
            &mut meandata,
            &edge_ids,
            &keep_routes,
        )
    });

    logger.log("fastdta routing", duration.as_nanos());

    let (_, duration) = measure(|| {
        prepare_next_iteration_for_sampled_routing(
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
            &keep_routes,
        );

        traffic_model_data.deconstruct(&input_dir).unwrap();

        if iteration == 0 {
            // initialize relative gap file with 0.0 for the first iteration
            append_relative_gap_to_file(0.0, &input_dir);
        } else {
            // get graph from previous iteration
            let (_, graph, cch) = get_graph_data_for_cch(input_dir, iteration);
            let customized_graph = customize(&cch, &graph);

            let (shortest_paths, _, departures) = get_paths_with_cch(&cch, &customized_graph, input_dir, &graph);

            let shortest_travel_times: Vec<FlWeight> = shortest_paths
                .iter()
                .enumerate()
                .map(|(i, path)| graph.get_travel_time_along_path(Timestamp::from_millis(departures[i]), path))
                .collect();

            let query_ids: Vec<String> = read_strings_from_file(&input_dir.join(FILE_QUERY_IDS)).unwrap();

            print_highest_differences(
                &shortest_travel_times.iter().map(|&tt| tt.into()).collect(),
                &travel_times.iter().map(|&tt| tt.into()).collect(),
                &paths,
                &previous_paths,
                &query_ids,
                &edge_ids,
            );

            set_relative_gap_with_previous_paths(&previous_paths, &graph, &input_dir, &shortest_travel_times, &departures);
        }
    });

    logger.log("postprocessing", duration.as_nanos());

    Ok(())
}

fn print_highest_differences(
    best_tts: &Vec<f64>,
    experienced_tts: &Vec<f64>,
    best_paths: &Vec<Vec<u32>>,
    experienced_paths: &Vec<&Vec<u32>>,
    query_ids: &Vec<String>,
    edge_ids: &Vec<String>,
) {
    let mut differences: Vec<(usize, f64)> = best_tts.par_iter().enumerate().map(|(i, &best_tt)| (i, experienced_tts[i] - best_tt)).collect();

    differences.par_sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("Top 20 highest differences between experienced and best travel times:");
    for i in 0..20.min(differences.len()) {
        let (index, diff) = differences[i];
        if diff > EPSILON_TRAVEL_TIME {
            println!(
                "Query ID: {}, Difference: {:.6}, \nBest TT: {:.6}, \nPath: {}\n Experienced TT: {:.6}, \nPath: {}",
                query_ids[index],
                diff,
                best_tts[index],
                get_path_ids_from_indices(edge_ids, &best_paths[index]).join(" "),
                experienced_tts[index],
                get_path_ids_from_indices(edge_ids, &experienced_paths[index]).join(" "),
            );
        }
    }
}

fn get_path_ids_from_indices(edge_ids: &Vec<String>, indices: &Vec<u32>) -> Vec<String> {
    indices.iter().map(|&i| edge_ids[i as usize].clone()).collect()
}
