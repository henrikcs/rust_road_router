use std::collections::HashMap;
use std::path::Path;

use conversion::sumo::FileReader;
use conversion::sumo::meandata_reader::SumoMeandataReader;
use conversion::sumo::routes::Vehicle;
use conversion::sumo::routes_reader::SumoRoutesReader;
use conversion::sumo::sumo_find_file::{get_meandata_file, get_routes_file_name_in_iteration};
use conversion::sumo::sumo_to_new_graph_weights::get_graph_with_travel_times_from_previous_iteration;
use conversion::{DIR_DTA, FILE_EDGE_INDICES_TO_ID};
use fastdta::alternative_path_assembler::assemble_alternative_paths;
use fastdta::alternative_paths::AlternativePathsForDTA;
use fastdta::cli;
use fastdta::cli::Parser;
use fastdta::customize::customize;
use fastdta::edge_occupancy::get_edge_occupancy_deltas;
use fastdta::preprocess::get_cch;
use fastdta::query::{get_paths_with_cch_queries, read_queries};
use rust_road_router::algo::catchup::Server;
use rust_road_router::datastr::graph::floating_time_dependent::Timestamp;
use rust_road_router::io::read_strings_from_file;
use rust_road_router::report::measure;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::RouterArgs::parse();

    let input_dir = Path::new(&args.input_dir);
    let input_prefix = &args.input_prefix;
    let iteration = args.iteration;

    let choice_algorithm = args.get_choice_algorithm();

    assert!(args.max_alternatives > 0, "max_alternatives must be greater than 0");

    let ((edge_ids, graph, cch, query_data, meandata, old_paths), duration) = measure(|| {
        let edge_ids: Vec<String> = read_strings_from_file(&input_dir.join(FILE_EDGE_INDICES_TO_ID)).unwrap_or_else(|_| {
            panic!(
                "Failed to read edge indices from file {} in directory {}",
                FILE_EDGE_INDICES_TO_ID,
                input_dir.display()
            )
        });
        let graph = get_graph_with_travel_times_from_previous_iteration(input_dir, iteration, &edge_ids);
        let cch = get_cch(input_dir, &graph);
        let query_data = read_queries(input_dir);
        let dta_iteration_dir = input_dir.join(format!("{:0>3}", iteration - 1)).join(DIR_DTA);

        // TODO: in `get_graph_with_travel_times_from_previous_iteration` we already read meandata; reuse it here
        let meandata = if iteration > 0 {
            Some(SumoMeandataReader::read(&get_meandata_file(&dta_iteration_dir)).expect("Failed to read SUMO meandata"))
        } else {
            None
        };

        if iteration == 0 {
            return (edge_ids, graph, cch, query_data, None, vec![Vec::new()]);
        }

        let alternative_paths = AlternativePathsForDTA::reconstruct(&dta_iteration_dir);

        let old_paths: Vec<Vec<u32>> = alternative_paths
            .alternatives_in_query
            .iter()
            .map(|ap| ap.paths[ap.choice].edges.clone())
            .collect();

        (edge_ids, graph, cch, query_data, meandata, old_paths)
    });
    log(&input_dir.display().to_string(), iteration, "preprocessing", duration.as_nanos());

    let (mut customized_graph, duration) = measure(|| customize(&cch, &graph));
    log(&input_dir.display().to_string(), iteration, "cch customization", duration.as_nanos());

    // customize with previous travel times
    // while not all trips have been sampled:
    //   sample a subset of trips
    //   find shortest routes for the sampled trips
    //   customize the graph with the shortest routes using the density on the edges during a time window

    // there should be two samples:
    // 1. every 10th query
    // 2. the remaining queries

    let samples: Vec<Vec<usize>> = vec![
        (0..query_data.0.len()).step_by(10).collect(),             // every 10th query
        (0..query_data.0.len()).filter(|i| i % 10 != 0).collect(), // the remaining queries
    ];

    let mut shortest_paths: Vec<Vec<u32>> = Vec::with_capacity(query_data.0.len());
    let mut travel_times = Vec::with_capacity(query_data.0.len());
    let mut departures = Vec::with_capacity(query_data.0.len());

    for (i, sample) in samples.iter().enumerate() {
        let ((sampled_shortest_paths, sampled_travel_times, sampled_departures), duration) = measure(|| {
            get_paths_with_cch_queries(
                &mut Server::new(&cch, &customized_graph),
                &sample.iter().map(|&i| query_data.0[i]).collect(),
                &sample.iter().map(|&i| query_data.1[i]).collect(),
                &sample.iter().map(|&i| query_data.2[i]).collect(),
                &sample.iter().map(|&i| query_data.3[i]).collect(),
                &sample.iter().map(|&i| query_data.4[i]).collect(),
                &graph,
            )
        });

        log(
            &input_dir.display().to_string(),
            iteration,
            format!("cch routing (sample {i})").as_str(),
            duration.as_nanos(),
        );

        let mut sampled_old_paths = Vec::with_capacity(sample.len());
        let mut sampled_departures_seconds = Vec::with_capacity(sample.len());

        sample.iter().for_each(|&i| {
            shortest_paths.insert(i, sampled_shortest_paths[i].clone());
            travel_times.insert(i, sampled_travel_times[i]);
            departures.insert(i, sampled_departures[i]);
            sampled_old_paths.push(old_paths.get(i).unwrap_or(&vec![]).clone());
            sampled_departures_seconds.push(Timestamp::from_millis(sampled_departures[i]));
        });

        let deltas = get_edge_occupancy_deltas(
            &graph,
            &sampled_old_paths,
            &sampled_shortest_paths,
            &sampled_departures_seconds,
            &vec![(0.0, 84600.0)], //TODO: extract from meandata
        );

        // TODO: apply deltas to meandata

        // TODO: use extract_interpolation_points_from_meandata to get the IPPs for the edges
        // TODO: create a new graph with the IPPs

        let (cg, duration) = measure(|| customize(&cch, &graph));
        customized_graph = cg;
        log(
            &input_dir.display().to_string(),
            iteration,
            format!("cch customization (sample {i})").as_str(),
            duration.as_nanos(),
        );
    }

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
            args.get_write_some_alternatives(),
            args.seed.unwrap_or(rand::random::<i32>()),
            &edge_ids,
        )
    });

    log(&input_dir.display().to_string(), iteration, "assembling alternative paths", duration.as_nanos());

    Ok(())
}

/// Logs the operation with the duration in nanoseconds within a certain iteration of certain run identified by identifier.
/// The format is: "sumo-fastdta-router; <identifier>; <iteration>; <operation>; <duration_in_nanos>"
fn log(identifier: &str, iteration: u32, operation: &str, duration_in_nanos: u128) {
    println!("sumo-fastdta-router; {}; {}; {}; {}", identifier, iteration, operation, duration_in_nanos);
}
