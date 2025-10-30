use std::path::Path;

use conversion::{
    DIR_DTA, FILE_EDGE_CAPACITIES, FILE_EDGE_DEFAULT_TRAVEL_TIMES, FILE_EDGE_INDICES_TO_ID, FILE_FIRST_IPP_OF_ARC, FILE_IPP_DEPARTURE_TIME,
    FILE_IPP_TRAVEL_TIME, SerializedTimestamp, SerializedTravelTime,
    sumo::{
        FileReader,
        meandata::MeandataDocumentRoot,
        meandata_reader::SumoMeandataReader,
        sumo_find_file::get_meandata_file,
        sumo_to_new_graph_weights::{extract_interpolation_points_from_meandata, get_graph_with_travel_times_from_previous_iteration},
    },
};
use rust_road_router::{
    algo::catchup::{Server, customize},
    datastr::graph::floating_time_dependent::Timestamp,
    io::{Reconstruct, Store},
};
use rust_road_router::{
    algo::customizable_contraction_hierarchy::CCH,
    datastr::graph::floating_time_dependent::{FlWeight, TDGraph},
    io::{Load, read_strings_from_file},
    report::measure,
};

use crate::{
    alternative_paths::AlternativePathsForDTA,
    edge_occupancy::get_edge_occupancy_deltas,
    logger::Logger,
    preprocess::get_cch,
    query::{get_paths_with_cch_queries, read_queries},
    vdf::{Bpr, Ptv, VDF, VDFType},
};

pub fn get_graph_data_for_fast_dta(
    input_dir: &Path,
    iteration: u32,
) -> (
    Vec<String>,
    (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>),
    Option<MeandataDocumentRoot>,
    Vec<Vec<u32>>,
) {
    let edge_ids = get_edge_ids(input_dir);
    let query_data = read_queries(input_dir);

    // TODO: in `get_graph_with_travel_times_from_previous_iteration` we already read meandata; reuse it here
    let meandata = if iteration > 0 {
        let iteration_dir = input_dir.join(format!("{:0>3}", iteration - 1));

        Some(SumoMeandataReader::read(&get_meandata_file(&iteration_dir)).expect("Failed to read SUMO meandata"))
    } else {
        None
    };

    if iteration == 0 {
        return (edge_ids, query_data, None, vec![Vec::new()]);
    }

    let iteration_dir = input_dir.join(format!("{:0>3}", iteration - 1));

    let alternative_paths = AlternativePathsForDTA::reconstruct(&iteration_dir.join(DIR_DTA));

    let old_paths: Vec<Vec<u32>> = alternative_paths
        .alternatives_in_query
        .iter()
        .map(|ap| ap.paths[ap.choice].edges.clone())
        .collect();

    (edge_ids, query_data, meandata, old_paths)
}

pub fn get_graph_data_for_cch(input_dir: &Path, iteration: u32) -> (Vec<String>, TDGraph, CCH) {
    let (edge_ids, graph) = get_graph_data_for_dijkstra(input_dir, iteration);
    let cch = get_cch(input_dir, &graph);

    (edge_ids, graph, cch)
}

pub fn get_graph_data_for_dijkstra(input_dir: &Path, iteration: u32) -> (Vec<String>, TDGraph) {
    let edge_ids: Vec<String> = get_edge_ids(input_dir);
    let graph = get_graph_with_travel_times_from_previous_iteration(input_dir, iteration, &edge_ids);

    (edge_ids, graph)
}

fn get_edge_ids(input_dir: &Path) -> Vec<String> {
    read_strings_from_file(&input_dir.join(FILE_EDGE_INDICES_TO_ID)).unwrap_or_else(|_| {
        panic!(
            "Failed to read edge indices from file {} in directory {}",
            FILE_EDGE_INDICES_TO_ID,
            input_dir.display()
        )
    })
}

pub fn get_paths_by_samples(
    input_dir: &Path,
    logger: &Logger,
    query_data: &(Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>),
    samples: &Vec<Vec<usize>>,
    vdf: VDFType,
    old_paths: &Vec<Vec<u32>>,
    meandata: &mut Option<MeandataDocumentRoot>,
    edge_ids: &Vec<String>,
) -> (TDGraph, Vec<Vec<u32>>, Vec<FlWeight>, Vec<SerializedTimestamp>) {
    // customize with previous travel times
    // while not all trips have been sampled:
    //   sample a subset of trips
    //   find shortest routes for the sampled trips
    //   customize the graph with the shortest routes using the density on the edges during a time window

    let mut shortest_paths: Vec<Vec<u32>> = vec![vec![]; query_data.0.len()]; // Vec::with_capacity(query_data.0.len());
    let mut travel_times = vec![FlWeight::INVALID; query_data.0.len()];
    let mut departures = vec![0; query_data.0.len()];
    let default_travel_times = &Vec::<SerializedTravelTime>::load_from(&input_dir.join(FILE_EDGE_DEFAULT_TRAVEL_TIMES)).unwrap();
    let edge_capas = &Vec::<f64>::load_from(&input_dir.join(FILE_EDGE_CAPACITIES)).unwrap();

    let periods = meandata.as_ref().map_or(vec![(0.0, 84600.0)], |md| {
        md.intervals.iter().map(|interval| (interval.begin, interval.end)).collect()
    });

    let mut graph: TDGraph = TDGraph::reconstruct_from(&input_dir).expect("Failed to reconstruct the time-dependent graph");

    for (i, sample) in samples.iter().enumerate() {
        let cch = get_cch(input_dir, &graph);
        let (customized_graph, duration) = measure(|| customize(&cch, &graph));

        logger.log(format!("cch customization (sample {i})").as_str(), duration.as_nanos());

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

        logger.log(format!("cch routing (sample {i})").as_str(), duration.as_nanos());

        let mut sampled_old_paths = Vec::with_capacity(sample.len());
        let mut sampled_departures_seconds = Vec::with_capacity(sample.len());

        sample.iter().enumerate().for_each(|(i, &query_i)| {
            shortest_paths[query_i] = sampled_shortest_paths[i].clone();
            travel_times[query_i] = sampled_travel_times[i];
            departures[query_i] = sampled_departures[i];
            sampled_old_paths.push(old_paths.get(query_i).unwrap_or(&vec![]).clone());
            sampled_departures_seconds.push(Timestamp::from_millis(sampled_departures[i]));
        });

        let deltas = get_edge_occupancy_deltas(&graph, &sampled_old_paths, &sampled_shortest_paths, &sampled_departures_seconds, &periods);

        if let Some(meandata) = meandata {
            // iterate over itervals in meandata, then apply deltas to the edges
            // from sampled_shortest_paths in the interval using edge_map

            for (i, interval) in meandata.intervals.iter_mut().enumerate() {
                for (edge_id, delta) in deltas[i].iter().enumerate() {
                    if *delta == 0.0 {
                        continue;
                    }
                    let interval_begin = interval.begin;
                    let interval_end = interval.end;
                    let period = interval_end - interval_begin;
                    let edge_name = &edge_ids[edge_id];
                    if let Some(edge) = interval.get_edge(edge_name) {
                        // keep sampled_seconds >= 0
                        let previous_flow = edge.get_traffic_volume(period);
                        let free_flow_tt = default_travel_times[edge_id] as f64 / 1000.0;
                        edge.sampled_seconds = Some(f64::max(edge.sampled_seconds.unwrap_or(0.0) + *delta, 0.0));

                        let vdf: Box<dyn VDF> = match &vdf {
                            VDFType::Ptv => Box::from(Ptv::create(-1, edge.speed.unwrap_or(0.0) as f64)),
                            VDFType::Bpr => Box::from(Bpr::create(0.15, 4.0)),
                        };

                        let previous_tt = edge.traveltime.unwrap_or(free_flow_tt);

                        edge.traveltime =
                            Some(vdf.travel_time_estimation(previous_flow, previous_tt, edge.get_traffic_volume(period), edge_capas[edge_id], free_flow_tt));

                        if edge_name == "a" || edge_name == "b" {
                            println!(
                                "Edge {} (capa: {}): tt updated from {:?} to {:?} (freeflow: {}) in interval {}-{}",
                                edge_name, edge_capas[edge_id], previous_tt, edge.traveltime, free_flow_tt, interval_begin, interval_end
                            );
                            println!(
                                "Edge {} (capa: {}): flow updated from {:?} to {:?} in interval {}-{}",
                                edge_name,
                                edge_capas[edge_id],
                                previous_flow,
                                edge.get_traffic_volume(period),
                                interval_begin,
                                interval_end
                            );
                        }
                    }
                }
            }

            let (first_ipp_of_arc, ipp_travel_time, ipp_departure_time) =
                extract_interpolation_points_from_meandata(&meandata, &edge_ids, default_travel_times);

            first_ipp_of_arc.write_to(&input_dir.join(FILE_FIRST_IPP_OF_ARC)).unwrap();
            ipp_travel_time.write_to(&input_dir.join(FILE_IPP_TRAVEL_TIME)).unwrap();
            ipp_departure_time.write_to(&input_dir.join(FILE_IPP_DEPARTURE_TIME)).unwrap();
        }

        graph = TDGraph::reconstruct_from(&input_dir).expect("Failed to reconstruct the time-dependent graph");
    }

    (graph, shortest_paths, travel_times, departures)
}
