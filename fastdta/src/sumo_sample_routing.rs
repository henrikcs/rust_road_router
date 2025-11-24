use std::path::Path;

use conversion::{
    FILE_EDGE_DEFAULT_TRAVEL_TIMES, FILE_QUERY_IDS, SerializedTimestamp, SerializedTravelTime,
    sumo::{FileReader, meandata_reader::SumoMeandataReader, sumo_to_new_graph_weights::extract_interpolation_points_from_meandata},
};

use rust_road_router::{
    algo::catchup::customize,
    datastr::graph::floating_time_dependent::{FlWeight, TDGraph},
    io::{Load, Reconstruct, read_strings_from_file},
    report::measure,
};

use crate::{
    logger::Logger,
    preprocess::get_cch,
    query::get_paths_with_cch_queries,
    sumo_routes_writer::write_batch_routes_for_sumo,
    sumo_runner::{SumoConfig, generate_additional_file, run_sumo},
};

/// Route all queries using samples, simulating each sample with SUMO
/// Returns the final graph, shortest paths, travel times, and departures
pub fn get_paths_by_samples_with_sumo(
    input_dir: &Path,
    net_file: &Path,
    iteration: u32,
    aggregation: u32,
    logger: &Logger,
    query_data: &(Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>),
    samples: &Vec<Vec<usize>>,
    edge_ids: &Vec<String>,
) -> (TDGraph, Vec<Vec<u32>>, Vec<FlWeight>, Vec<SerializedTimestamp>) {
    let mut shortest_paths: Vec<Vec<u32>> = vec![vec![]; query_data.0.len()];
    let mut travel_times = vec![FlWeight::INVALID; query_data.0.len()];
    let mut departures = vec![0; query_data.0.len()];

    let free_flow_tts_ms = &Vec::<SerializedTravelTime>::load_from(&input_dir.join(FILE_EDGE_DEFAULT_TRAVEL_TIMES)).unwrap();
    let query_ids: Vec<String> = read_strings_from_file(&input_dir.join(FILE_QUERY_IDS)).unwrap();

    // Start with the base graph (either from previous iteration or free-flow)
    let mut graph: TDGraph = TDGraph::reconstruct_from(&input_dir).expect("Failed to reconstruct the time-dependent graph");

    // If iteration > 0, load previous iteration's meandata to initialize the graph
    if iteration > 0 {
        let previous_iteration_dir = input_dir.join(format!("{:0>3}", iteration - 1));
        let previous_meandata_file = find_latest_meandata_file(&previous_iteration_dir);

        if let Some(meandata_file) = previous_meandata_file {
            let meandata = SumoMeandataReader::read(&meandata_file).expect("Failed to read previous SUMO meandata");
            let (first_ipp_of_arc, ipp_travel_time, ipp_departure_time) = extract_interpolation_points_from_meandata(&meandata, &edge_ids, &free_flow_tts_ms);

            graph = TDGraph::new(
                Vec::from(graph.first_out()),
                Vec::from(graph.head()),
                first_ipp_of_arc,
                ipp_departure_time,
                ipp_travel_time,
            );
        }
    }

    let cch = get_cch(input_dir, &graph);

    // Keep track of all routed paths for SUMO simulation
    let mut all_routed_paths: Vec<Vec<u32>> = Vec::new();
    let mut all_routed_trip_ids: Vec<String> = Vec::new();
    let mut all_routed_departures: Vec<SerializedTimestamp> = Vec::new();

    for (batch_idx, sample) in samples.iter().enumerate() {
        logger.log(&format!("Processing batch {}/{}", batch_idx + 1, samples.len()), 0);

        // Customize and route current sample
        let (customized_graph, duration) = measure(|| customize(&cch, &graph));
        logger.log(&format!("cch customization (batch {batch_idx})"), duration.as_nanos());

        let ((sampled_shortest_paths, sampled_travel_times, sampled_departures), duration) = measure(|| {
            get_paths_with_cch_queries(
                &cch,
                &customized_graph,
                &sample.iter().map(|&i| query_data.0[i]).collect(),
                &sample.iter().map(|&i| query_data.1[i]).collect(),
                &sample.iter().map(|&i| query_data.2[i]).collect(),
                &sample.iter().map(|&i| query_data.3[i]).collect(),
                &sample.iter().map(|&i| query_data.4[i]).collect(),
                &graph,
            )
        });
        logger.log(&format!("routing (batch {batch_idx})"), duration.as_nanos());

        // Store results for this sample
        sample.iter().enumerate().for_each(|(i, &query_i)| {
            shortest_paths[query_i] = sampled_shortest_paths[i].clone();
            travel_times[query_i] = sampled_travel_times[i];
            departures[query_i] = sampled_departures[i];
        });

        // Add current batch to accumulated routes for SUMO simulation
        for (i, &query_i) in sample.iter().enumerate() {
            all_routed_paths.push(sampled_shortest_paths[i].clone());
            all_routed_trip_ids.push(query_ids[query_i].clone());
            all_routed_departures.push(sampled_departures[i]);
        }

        // Run SUMO simulation with all routes up to this point
        let (_, duration) = measure(|| {
            run_sumo_simulation_for_batch(
                input_dir,
                net_file,
                iteration,
                batch_idx,
                aggregation,
                &all_routed_paths,
                &all_routed_trip_ids,
                &all_routed_departures,
                edge_ids,
            )
            .expect("Failed to run SUMO simulation");
        });
        logger.log(&format!("sumo simulation (batch {batch_idx})"), duration.as_nanos());

        // Read the dump file and update graph weights
        let (_, duration) = measure(|| {
            update_graph_from_sumo_dump(input_dir, iteration, batch_idx, aggregation, &mut graph, edge_ids, free_flow_tts_ms)
                .expect("Failed to update graph from SUMO dump");
        });
        logger.log(&format!("update graph weights (batch {batch_idx})"), duration.as_nanos());
    }

    (graph, shortest_paths, travel_times, departures)
}

/// Run SUMO simulation for a batch
fn run_sumo_simulation_for_batch(
    input_dir: &Path,
    net_file: &Path,
    iteration: u32,
    batch: usize,
    aggregation: u32,
    paths: &Vec<Vec<u32>>,
    trip_ids: &Vec<String>,
    departures: &Vec<SerializedTimestamp>,
    edge_ids: &Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let current_iteration_dir = input_dir.join(format!("{:0>3}", iteration));
    std::fs::create_dir_all(&current_iteration_dir)?;

    // Write routes file
    let routes_file = current_iteration_dir.join(format!("routes_batch_{:0>3}.rou.xml", batch));
    write_batch_routes_for_sumo(&routes_file, trip_ids, paths, departures, edge_ids)?;

    // Generate additional file
    let additional_file = current_iteration_dir.join(format!("additional_batch_{:0>3}.xml", batch));
    generate_additional_file(&additional_file, aggregation, iteration, batch as u32)?;

    // Run SUMO
    let config = SumoConfig::new(net_file.to_path_buf(), routes_file, additional_file);

    run_sumo(&config)?;

    Ok(())
}

/// Update graph weights from SUMO dump file
fn update_graph_from_sumo_dump(
    input_dir: &Path,
    iteration: u32,
    batch: usize,
    aggregation: u32,
    graph: &mut TDGraph,
    edge_ids: &Vec<String>,
    free_flow_tts_ms: &Vec<SerializedTravelTime>,
) -> Result<(), Box<dyn std::error::Error>> {
    let current_iteration_dir = input_dir.join(format!("{:0>3}", iteration));
    let dump_file = current_iteration_dir.join(format!("dump_{}_{:0>3}_{:0>3}.xml", aggregation, iteration, batch));

    // Read meandata from dump file
    let meandata = SumoMeandataReader::read(&dump_file)?;

    // Extract interpolation points and update graph
    let (first_ipp_of_arc, ipp_travel_time, ipp_departure_time) = extract_interpolation_points_from_meandata(&meandata, edge_ids, free_flow_tts_ms);

    *graph = TDGraph::new(
        Vec::from(graph.first_out()),
        Vec::from(graph.head()),
        first_ipp_of_arc,
        ipp_departure_time,
        ipp_travel_time,
    );

    Ok(())
}

/// Find the latest meandata file in a directory (highest batch number)
fn find_latest_meandata_file(dir: &Path) -> Option<std::path::PathBuf> {
    if !dir.exists() {
        return None;
    }

    let mut dump_files: Vec<_> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() && path.file_name()?.to_str()?.starts_with("dump_") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    dump_files.sort();
    dump_files.last().cloned()
}
