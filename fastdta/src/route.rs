use std::{default, path::Path};

use conversion::{
    DIR_DTA, FILE_EDGE_CAPACITIES, FILE_EDGE_DEFAULT_TRAVEL_TIMES, FILE_EDGE_INDICES_TO_ID, FILE_EDGE_LANES, FILE_EDGE_LENGTHS, SerializedTimestamp,
    SerializedTravelTime,
    sumo::{
        FileReader, FileWriter,
        meandata::{Interval, MeandataDocumentRoot},
        meandata_reader::SumoMeandataReader,
        meandata_writer::SumoMeandataWriter,
        sumo_find_file::get_meandata_file,
        sumo_to_new_graph_weights::{extract_interpolation_points_from_meandata, get_graph_with_travel_times_from_previous_iteration},
    },
};

use rust_road_router::{
    algo::{
        catchup::customize,
        customizable_contraction_hierarchy::{CCH, CCHT},
    },
    datastr::graph::floating_time_dependent::{EPSILON, FlWeight, Shortcut, ShortcutGraphTrt, ShortcutId, TDGraph, TDGraphTrait},
    io::{Load, read_strings_from_file},
    report::measure,
};
use rust_road_router::{datastr::graph::floating_time_dependent::Timestamp, io::Reconstruct};

use crate::{
    alternative_paths::AlternativePathsForDTA,
    edge_occupancy::get_edge_occupancy_deltas,
    logger::Logger,
    preprocess::get_cch,
    query::{get_paths_with_cch_queries, get_paths_with_dijkstra_queries, read_queries},
    vdf::{Bpr, Ptv, VDF, VDFType},
};

pub fn get_graph_data_for_fast_dta(
    input_dir: &Path,
    iteration: u32,
) -> (
    Vec<String>,
    (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>),
    MeandataDocumentRoot,
    AlternativePathsForDTA,
) {
    let edge_ids = get_edge_ids(input_dir);
    let query_data = read_queries(input_dir);

    // TODO: in `get_graph_with_travel_times_from_previous_iteration` we already read meandata; reuse it here
    let meandata = if iteration > 0 {
        let iteration_dir = input_dir.join(format!("{:0>3}", iteration - 1));

        SumoMeandataReader::read(&get_meandata_file(&iteration_dir)).expect("Failed to read SUMO meandata")
    } else {
        MeandataDocumentRoot::empty()
    };

    if iteration == 0 {
        return (
            edge_ids,
            query_data,
            MeandataDocumentRoot::empty(),
            AlternativePathsForDTA::init(&vec![], &vec![]),
        );
    }

    let iteration_dir = input_dir.join(format!("{:0>3}", iteration - 1));

    let alternative_paths = AlternativePathsForDTA::reconstruct(&iteration_dir.join(DIR_DTA));

    (edge_ids, query_data, meandata, alternative_paths)
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
    iteration: u32,
    logger: &Logger,
    query_data: &(Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>),
    samples: &Vec<Vec<usize>>,
    vdf: VDFType,
    alternative_paths_from_dta: &AlternativePathsForDTA,
    meandata: &mut MeandataDocumentRoot,
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
    let free_flow_tts_ms = &Vec::<SerializedTravelTime>::load_from(&input_dir.join(FILE_EDGE_DEFAULT_TRAVEL_TIMES)).unwrap();
    let free_flow_tts: Vec<f64> = free_flow_tts_ms.iter().map(|&tt| tt as f64 / 1000.0).collect();
    let edge_capas = &Vec::<f64>::load_from(&input_dir.join(FILE_EDGE_CAPACITIES)).unwrap();
    let edge_lengths = &Vec::<f64>::load_from(&input_dir.join(FILE_EDGE_LENGTHS)).unwrap();
    let edge_lanes = &Vec::<u32>::load_from(&input_dir.join(FILE_EDGE_LANES)).unwrap();

    let mut graph: TDGraph = TDGraph::reconstruct_from(&input_dir).expect("Failed to reconstruct the time-dependent graph");

    let old_paths = alternative_paths_from_dta.get_chosen_paths();

    for (i, sample) in samples.iter().enumerate() {
        let (first_ipp_of_arc, ipp_travel_time, ipp_departure_time) = extract_interpolation_points_from_meandata(&meandata, &edge_ids, &free_flow_tts_ms);

        graph = TDGraph::new(
            Vec::from(graph.first_out()),
            Vec::from(graph.head()),
            first_ipp_of_arc,
            ipp_departure_time,
            ipp_travel_time,
        );
        let cch = get_cch(input_dir, &graph);
        let (customized_graph, duration) = measure(|| customize(&cch, &graph));

        // logger.log(format!("cch customization (sample {i})").as_str(), duration.as_nanos());

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

        let mut sampled_old_paths: Vec<&Vec<u32>> = Vec::with_capacity(sample.len());
        let empty_vec: Vec<u32> = Vec::new();
        let mut sampled_departures_seconds = Vec::with_capacity(sample.len());

        sample.iter().enumerate().for_each(|(i, &query_i)| {
            shortest_paths[query_i] = sampled_shortest_paths[i].clone();
            travel_times[query_i] = sampled_travel_times[i];
            departures[query_i] = sampled_departures[i];
            sampled_old_paths.push(*old_paths.get(query_i).unwrap_or(&&empty_vec));
            sampled_departures_seconds.push(Timestamp::from_millis(sampled_departures[i]));
        });

        let deltas = get_edge_occupancy_deltas(
            &mut graph,
            &sampled_old_paths,
            &sampled_shortest_paths,
            &sampled_departures_seconds,
            &mut meandata.intervals,
            edge_ids,
            edge_lengths,
            &free_flow_tts,
            edge_lanes,
        );

        println!("Applying edge occupancy deltas for sample {i}: {:?}", deltas);

        debug(&meandata, &input_dir, iteration as u32, i as u32);
        // iterate over itervals in meandata, then apply deltas to the edges
        // from sampled_shortest_paths in the interval using edge_map

        // for (i, interval) in meandata.intervals.iter_mut().enumerate() {
        //     for (edge_id, delta) in deltas[i].iter().enumerate() {
        //         if delta.abs() < EPSILON {
        //             continue;
        //         }
        //         let interval_begin = interval.begin;
        //         let interval_end = interval.end;
        //         let period = interval_end - interval_begin;
        //         let edge_name = &edge_ids[edge_id];
        //         if let Some(edge) = interval.get_edge(edge_name) {
        //             let previous_flow: f64 = edge.get_traffic_volume(period);
        //             let free_flow_tt = default_travel_times[edge_id] as f64 / 1000.0;
        //             let previous_tt = edge.traveltime.unwrap_or(free_flow_tt);
        //             let previous_density = edge.density.unwrap_or(0.0);

        //             edge.sampled_seconds = Some(f64::max(edge.sampled_seconds.unwrap_or(0.0) + *delta, 0.0));

        //             let vdf: Box<dyn VDF> = match &vdf {
        //                 VDFType::Ptv => Box::from(Ptv::create(-1, edge.speed.unwrap_or(0.0) as f64)),
        //                 VDFType::Bpr => Box::from(Bpr::create(0.15, 4.0)),
        //             };

        //             let new_flow = edge.get_traffic_volume(period);

        //             // set capacity of edge
        //             let capacity = if previous_flow > 0.0 {
        //                 estimate_capacity_bpr(previous_tt, previous_flow, free_flow_tt)
        //             } else {
        //                 edge_capas[edge_id]
        //             };

        //             let length = edge_lengths[edge_id];
        //             // esitmate new density
        //             let new_density = edge.sampled_seconds.unwrap_or(0.0) / (period * length) * 1000.0;

        //             edge.traveltime = Some(vdf.travel_time_estimation(
        //                 previous_flow,
        //                 previous_density,
        //                 previous_tt,
        //                 new_flow,
        //                 new_density,
        //                 capacity,
        //                 length,
        //                 free_flow_tt,
        //             ));

        //             edge.density = Some(new_density);

        //             // if (edge_name == "a" || edge_name == "b") && i == 0 {
        //             //     println!(
        //             //         "Edge {} (capa: {}): tt updated from {:?} to {:?} (freeflow: {}) in interval {}-{}",
        //             //         edge_name, capacity, previous_tt, edge.traveltime, free_flow_tt, interval_begin, interval_end
        //             //     );
        //             //     println!(
        //             //         "Edge {} (capa: {}): flow updated from {:?} to {:?} in interval {}-{}",
        //             //         edge_name, capacity, previous_flow, new_flow, interval_begin, interval_end
        //             //     );
        //             // }
        //         }
        //     }
        // }
    }

    (graph, shortest_paths, travel_times, departures)
}

fn estimate_capacity_bpr(previous_tt: f64, previous_flow: f64, free_flow_tt: f64) -> f64 {
    if previous_tt <= free_flow_tt {
        return f64::INFINITY;
    }

    let alpha = 0.15;
    let beta = 4.0;

    let capacity = previous_flow * (free_flow_tt / ((previous_tt - free_flow_tt) * alpha)).powf(-1.0 / beta);

    capacity
}

fn debug(meandata: &MeandataDocumentRoot, path: &Path, iteration: u32, sample: u32) {
    let output_file = path.join(format!("dump_i_{:0>3}_s_{:0>3}.xml", iteration, sample));

    SumoMeandataWriter::write(&output_file, meandata).unwrap();
}
