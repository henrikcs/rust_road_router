use std::path::Path;

use conversion::{
    FILE_EDGE_DEFAULT_TRAVEL_TIMES, FILE_EDGE_LANES, FILE_EDGE_LENGTHS, SerializedTimestamp, SerializedTravelTime,
    sumo::{
        FileWriter, meandata::MeandataDocumentRoot, meandata_writer::SumoMeandataWriter, sumo_to_new_graph_weights::extract_interpolation_points_from_meandata,
    },
};

use rust_road_router::{
    algo::{catchup::customize, customizable_contraction_hierarchy::CCH},
    datastr::graph::floating_time_dependent::{CustomizedGraph, FlWeight, TDGraph},
    io::Load,
    report::measure,
};
use rust_road_router::{datastr::graph::floating_time_dependent::Timestamp, io::Reconstruct};

use crate::{
    logger::Logger, path_processor::adjust_weights_in_graph_by_following_paths, preprocess::get_cch, query::get_paths_with_cch_queries,
    traffic_model::TrafficModel,
};

pub fn get_paths_by_samples_with_keep_routes(
    input_dir: &Path,
    _iteration: u32,
    logger: &Logger,
    query_data: &(Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>),
    samples: &Vec<Vec<usize>>,
    traffic_models: &Vec<Box<dyn TrafficModel>>,
    previous_paths: &Vec<&Vec<u32>>,
    meandata: &mut MeandataDocumentRoot,
    edge_ids: &Vec<String>,
    keep_routes: &Vec<bool>,
) -> (TDGraph, Vec<Vec<u32>>, Vec<FlWeight>, Vec<SerializedTimestamp>) {
    // customize with previous travel times
    // while not all trips have been sampled:
    //   sample a subset of trips
    //   find shortest routes for the sampled trips
    //   customize the graph with the shortest routes using the density on the edges during a time window

    let mut routed_paths: Vec<Vec<u32>> = previous_paths.iter().map(|path| (*path).clone()).collect();
    let mut routed_paths_tt = vec![FlWeight::INVALID; query_data.0.len()];
    let departures = query_data.2.clone();

    let free_flow_tts_ms = &Vec::<SerializedTravelTime>::load_from(&input_dir.join(FILE_EDGE_DEFAULT_TRAVEL_TIMES)).unwrap();
    let free_flow_tts: Vec<f64> = free_flow_tts_ms.iter().map(|&tt| tt as f64 / 1000.0).collect();

    let edge_lengths = &Vec::<f64>::load_from(&input_dir.join(FILE_EDGE_LENGTHS)).unwrap();
    let edge_lanes = &Vec::<u32>::load_from(&input_dir.join(FILE_EDGE_LANES)).unwrap();

    let mut graph: TDGraph = TDGraph::reconstruct_from(&input_dir).expect("Failed to reconstruct the time-dependent graph");
    let original_meandata = meandata.clone();
    let (first_ipp_of_arc, ipp_travel_time, ipp_departure_time) = extract_interpolation_points_from_meandata(&meandata, &edge_ids, &free_flow_tts_ms);

    graph = TDGraph::new(
        Vec::from(graph.first_out()),
        Vec::from(graph.head()),
        first_ipp_of_arc,
        ipp_departure_time,
        ipp_travel_time,
    );
    let cch = get_cch(input_dir, &graph);

    for (i, sample) in samples.iter().enumerate() {
        let (customized_graph, duration) = measure(|| customize(&cch, &graph));

        logger.log(format!("cch customization (sample {i})").as_str(), duration.as_nanos());

        let (sampled_new_paths, duration) =
            measure(|| get_sampled_queries_with_keep_routes(&graph, &cch, &customized_graph, keep_routes, sample, query_data, previous_paths));

        logger.log(format!("routing (sample {i})").as_str(), duration.as_nanos());

        let mut sampled_old_paths: Vec<&Vec<u32>> = Vec::with_capacity(sample.len());

        sample.iter().enumerate().for_each(|(i, &query_i)| {
            routed_paths[query_i] = sampled_new_paths[i].clone();
            routed_paths_tt[query_i] = graph.get_travel_time_along_path(Timestamp::from_millis(departures[query_i]), &sampled_new_paths[i]);
            sampled_old_paths.push(previous_paths[query_i]);
        });

        adjust_weights_in_graph_by_following_paths(
            &mut graph,
            &sampled_old_paths,
            &sampled_new_paths,
            &departures.iter().map(|&d| Timestamp::from_millis(d)).collect(),
            &mut meandata.intervals,
            edge_ids,
            edge_lengths,
            &free_flow_tts,
            &traffic_models,
            &edge_lanes,
        );

        // println!("Applying edge occupancy deltas for sample {i}: {:?}", deltas);

        // debug(&meandata, &input_dir, iteration as u32, i as u32);
    }

    let (first_ipp_of_arc, ipp_travel_time, ipp_departure_time) = extract_interpolation_points_from_meandata(&original_meandata, &edge_ids, &free_flow_tts_ms);

    graph = TDGraph::new(
        Vec::from(graph.first_out()),
        Vec::from(graph.head()),
        first_ipp_of_arc,
        ipp_departure_time,
        ipp_travel_time,
    );

    (graph, routed_paths, routed_paths_tt, departures)
}

fn _debug(meandata: &MeandataDocumentRoot, path: &Path, iteration: u32, sample: u32) {
    let output_file = path.join(format!("dump_i_{:0>3}_s_{:0>3}.xml", iteration, sample));

    SumoMeandataWriter::write(&output_file, meandata).unwrap();
}

pub fn get_sampled_queries_with_keep_routes(
    graph: &TDGraph,
    cch: &CCH,
    customized_graph: &CustomizedGraph,
    keep_routes: &Vec<bool>,
    sample: &Vec<usize>,
    query_data: &(Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>),
    previous_paths: &Vec<&Vec<u32>>,
) -> Vec<Vec<u32>> {
    // queries which are not reroutable keep their previous paths
    // for other queries, find new shortest paths
    // in the end, return the combined set of paths
    let reroutable_samples: Vec<&usize> = sample.iter().filter(|&i| !keep_routes[*i]).collect();

    let (rerouted_paths, _, _) = get_paths_with_cch_queries(
        &cch,
        &customized_graph,
        &reroutable_samples.iter().map(|&i| query_data.0[*i]).collect(),
        &reroutable_samples.iter().map(|&i| query_data.1[*i]).collect(),
        &reroutable_samples.iter().map(|&i| query_data.2[*i]).collect(),
        &reroutable_samples.iter().map(|&i| query_data.3[*i]).collect(),
        &reroutable_samples.iter().map(|&i| query_data.4[*i]).collect(),
        &graph,
    );

    // combine new paths with previous paths for non-reroutable queries

    let mut rerouted_path_index = 0;
    let sampled_routes: Vec<Vec<u32>> = sample
        .iter()
        .map(|s| {
            if keep_routes[*s] {
                previous_paths[*s].clone()
            } else {
                let p = rerouted_paths[rerouted_path_index].clone();
                rerouted_path_index += 1;
                p
            }
        })
        .collect();

    sampled_routes
}

pub fn get_paths_by_samples(
    input_dir: &Path,
    _iteration: u32,
    logger: &Logger,
    query_data: &(Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>),
    samples: &Vec<Vec<usize>>,
    traffic_models: &Vec<Box<dyn TrafficModel>>,
    previous_paths: &Vec<&Vec<u32>>,
    meandata: &mut MeandataDocumentRoot,
    edge_ids: &Vec<String>,
) -> (TDGraph, Vec<Vec<u32>>, Vec<FlWeight>, Vec<SerializedTimestamp>) {
    // customize with previous travel times
    // while not all trips have been sampled:
    //   sample a subset of trips
    //   find shortest routes for the sampled trips
    //   customize the graph with the shortest routes using the density on the edges during a time window

    let mut routed_paths: Vec<Vec<u32>> = vec![vec![]; query_data.0.len()]; // Vec::with_capacity(query_data.0.len());
    let mut travel_times = vec![FlWeight::INVALID; query_data.0.len()];
    let mut departures = vec![0; query_data.0.len()];
    let free_flow_tts_ms = &Vec::<SerializedTravelTime>::load_from(&input_dir.join(FILE_EDGE_DEFAULT_TRAVEL_TIMES)).unwrap();
    let free_flow_tts: Vec<f64> = free_flow_tts_ms.iter().map(|&tt| tt as f64 / 1000.0).collect();

    let edge_lengths = &Vec::<f64>::load_from(&input_dir.join(FILE_EDGE_LENGTHS)).unwrap();
    let edge_lanes = &Vec::<u32>::load_from(&input_dir.join(FILE_EDGE_LANES)).unwrap();

    let mut graph: TDGraph = TDGraph::reconstruct_from(&input_dir).expect("Failed to reconstruct the time-dependent graph");
    let original_meandata = meandata.clone();
    let (first_ipp_of_arc, ipp_travel_time, ipp_departure_time) = extract_interpolation_points_from_meandata(&meandata, &edge_ids, &free_flow_tts_ms);

    graph = TDGraph::new(
        Vec::from(graph.first_out()),
        Vec::from(graph.head()),
        first_ipp_of_arc,
        ipp_departure_time,
        ipp_travel_time,
    );
    let cch = get_cch(input_dir, &graph);

    for (i, sample) in samples.iter().enumerate() {
        let (customized_graph, duration) = measure(|| customize(&cch, &graph));

        logger.log(format!("cch customization (sample {i})").as_str(), duration.as_nanos());

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

        logger.log(format!("routing (sample {i})").as_str(), duration.as_nanos());

        let (_, duration) = measure(|| {
            let mut sampled_old_paths: Vec<&Vec<u32>> = Vec::with_capacity(sample.len());
            let empty_vec: Vec<u32> = Vec::new();
            let mut sampled_departures_seconds = Vec::with_capacity(sample.len());

            sample.iter().enumerate().for_each(|(i, &query_i)| {
                routed_paths[query_i] = sampled_shortest_paths[i].clone();
                travel_times[query_i] = sampled_travel_times[i];
                departures[query_i] = sampled_departures[i];
                sampled_old_paths.push(*previous_paths.get(query_i).unwrap_or(&&empty_vec));
                sampled_departures_seconds.push(Timestamp::from_millis(sampled_departures[i]));
            });

            adjust_weights_in_graph_by_following_paths(
                &mut graph,
                &sampled_old_paths,
                &sampled_shortest_paths,
                &sampled_departures_seconds,
                &mut meandata.intervals,
                edge_ids,
                edge_lengths,
                &free_flow_tts,
                &traffic_models,
                &edge_lanes,
            );
        });
        logger.log(format!("adjust weights (sample {i})").as_str(), duration.as_nanos());

        // println!("Applying edge occupancy deltas for sample {i}: {:?}", deltas);

        // debug(&meandata, &input_dir, iteration as u32, i as u32);
    }

    let (first_ipp_of_arc, ipp_travel_time, ipp_departure_time) = extract_interpolation_points_from_meandata(&original_meandata, &edge_ids, &free_flow_tts_ms);

    graph = TDGraph::new(
        Vec::from(graph.first_out()),
        Vec::from(graph.head()),
        first_ipp_of_arc,
        ipp_departure_time,
        ipp_travel_time,
    );

    (graph, routed_paths, travel_times, departures)
}
