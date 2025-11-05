use std::{collections::HashMap, path::Path};

use rayon::prelude::*;

use rust_road_router::{
    datastr::graph::{floating_time_dependent::TDGraph, EdgeId},
    io::{Load, Reconstruct, Store},
};

use crate::{
    sumo::{
        meandata::{Edge, MeandataDocumentRoot},
        meandata_reader::SumoMeandataReader,
        sumo_find_file::get_meandata_file,
        sumo_to_td_graph_converter::MIN_EDGE_WEIGHT,
        FileReader,
    },
    SerializedTimestamp, SerializedTravelTime, FILE_EDGE_DEFAULT_TRAVEL_TIMES, FILE_FIRST_IPP_OF_ARC, FILE_IPP_DEPARTURE_TIME, FILE_IPP_TRAVEL_TIME,
};

pub fn get_graph_with_travel_times_from_previous_iteration(input_dir: &Path, iteration: u32, edge_ids: &Vec<String>) -> TDGraph {
    // if iteration > 0, we load the previous iteration's travel times
    if iteration > 0 {
        let previous_iteration_dir = input_dir.join(format!("{:0>3}", iteration - 1));

        extract_travel_times_from_iteration_directory(&previous_iteration_dir, &input_dir, &edge_ids);
    }

    // TODO: instead of reconstructing the graph from a file, we could create it in memory
    TDGraph::reconstruct_from(&input_dir).expect("Failed to reconstruct the time-dependent graph")
}

pub fn extract_travel_times_from_iteration_directory(iteration_dir: &Path, path_to_graph_weights: &Path, edge_indices_to_id: &Vec<String>) {
    // dump file starts with "dump_" and ends with ".xml"
    let dump_file = get_meandata_file(&iteration_dir);

    set_new_graph_weights_from_meandata_file(
        &path_to_graph_weights,
        &dump_file,
        &edge_indices_to_id,
        &Vec::<SerializedTravelTime>::load_from(path_to_graph_weights.join(FILE_EDGE_DEFAULT_TRAVEL_TIMES)).unwrap(),
    );
}

pub fn set_new_graph_weights_from_meandata_file(
    path_to_graph_weights: &Path,
    path_to_sumo_meandata: &Path,
    edge_indices_to_id: &Vec<String>,
    edge_default_travel_times: &Vec<SerializedTravelTime>,
) {
    let meandata = SumoMeandataReader::read(path_to_sumo_meandata).expect("Failed to read SUMO meandata");

    let (first_ipp_of_arc, ipp_travel_time, ipp_departure_time) =
        extract_interpolation_points_from_meandata(&meandata, &edge_indices_to_id, &edge_default_travel_times);

    first_ipp_of_arc.write_to(&path_to_graph_weights.join(FILE_FIRST_IPP_OF_ARC)).unwrap();
    ipp_travel_time.write_to(&path_to_graph_weights.join(FILE_IPP_TRAVEL_TIME)).unwrap();
    ipp_departure_time.write_to(&path_to_graph_weights.join(FILE_IPP_DEPARTURE_TIME)).unwrap();
}

pub fn extract_interpolation_points_from_meandata(
    meandata: &MeandataDocumentRoot,
    edge_indices_to_id: &Vec<String>,
    edge_default_travel_times: &Vec<SerializedTravelTime>,
) -> (Vec<EdgeId>, Vec<SerializedTravelTime>, Vec<SerializedTimestamp>) {
    get_ipp_vectors(
        &meandata,
        edge_indices_to_id,
        &preprocess_tt(&meandata, edge_indices_to_id, edge_default_travel_times),
    )
}

fn preprocess_tt<'a>(
    meandata: &MeandataDocumentRoot,
    edge_indices_to_id: &'a Vec<String>,
    edge_default_travel_times: &Vec<SerializedTravelTime>,
) -> HashMap<&'a String, HashMap<SerializedTimestamp, SerializedTravelTime>> {
    let mut original_edge_by_edge_id_and_interval: HashMap<&String, HashMap<SerializedTimestamp, &Edge>> = HashMap::with_capacity(edge_indices_to_id.len());

    for interval in &meandata.intervals {
        // for each interval, create a map of edge id to edge
        let timestamp = (interval.begin * 1000.0) as SerializedTimestamp;

        for edge in &interval.edges {
            original_edge_by_edge_id_and_interval
                .entry(&edge.id)
                .or_insert_with(HashMap::new)
                .insert(timestamp, edge);
        }
    }

    edge_indices_to_id
        .par_iter()
        .enumerate()
        .map(|(edge_index, edge_id)| {
            let mut adapted_tt: HashMap<SerializedTimestamp, SerializedTravelTime> = HashMap::new();
            let edge_tts = match original_edge_by_edge_id_and_interval.get(edge_id) {
                Some(e) => e,
                None => &HashMap::new(),
            };
            let default_travel_time = edge_default_travel_times[edge_index];

            for (interval_index, interval) in meandata.intervals.iter().rev().enumerate() {
                let timestamp = (interval.begin * 1000.0) as SerializedTimestamp;

                let mut tt = edge_tts
                    .get(&timestamp)
                    .and_then(|edge| edge.traveltime)
                    // should not go lower than the default travel time
                    .map(|val| (u32::max((val * 1000.0) as u32, default_travel_time)) as SerializedTravelTime)
                    .unwrap_or(default_travel_time);

                if interval_index == 0 {
                    // in the last interval, cut the travel time to length of the last interval + the minimum travel time of the edge
                    // this ensures the fifo property for the travel time in the last interval
                    // since the time functions are periodic and wrap around
                    let interval_duration = ((interval.end - interval.begin) * 1000.0) as SerializedTravelTime;

                    if interval_duration + default_travel_time < tt {
                        // If the next travel time is less than the current, we adjust the current travel time
                        println!(
                            "Adjusting travel time for edge {} in interval {}-{}: {}ms -> {} + {} = {}ms",
                            edge_id,
                            timestamp,
                            0,
                            tt,
                            interval_duration,
                            default_travel_time,
                            interval_duration + default_travel_time
                        );
                        tt = interval_duration + default_travel_time;
                    }
                }
                if interval_index > 0 {
                    // Enforce FIFO condition if there is a next interval
                    if let Some(next_interval) = meandata.intervals.get(meandata.intervals.len() - interval_index) {
                        let next_timestamp = (next_interval.begin * 1000.0) as SerializedTimestamp;
                        let interval_duration = next_timestamp - timestamp;
                        let next_tt = adapted_tt.get(&next_timestamp).unwrap();

                        if interval_duration + next_tt < tt {
                            // If the next travel time is less than the current, we adjust the current travel time
                            println!(
                                "Adjusting travel time for edge {} in interval {}-{}: {}ms -> {} + {} = {}ms",
                                edge_id,
                                timestamp,
                                next_timestamp,
                                tt,
                                interval_duration,
                                next_tt,
                                interval_duration + next_tt
                            );
                            tt = interval_duration + next_tt;
                        }
                    }
                }

                if tt < (MIN_EDGE_WEIGHT * 1000.0) as SerializedTravelTime {
                    println!(
                        "Min Edge Weight Warning: Travel time for edge {} in interval starting at {} is less than MIN_EDGE_WEIGHT ({}ms). Adjusting to {}ms.",
                        edge_id, timestamp, tt, MIN_EDGE_WEIGHT
                    );
                    tt = (MIN_EDGE_WEIGHT * 1000.0) as SerializedTravelTime;
                }
                adapted_tt.insert(timestamp, tt as SerializedTravelTime);
            }

            (edge_id, adapted_tt)
        })
        .collect()
}

fn get_ipp_vectors(
    meandata: &MeandataDocumentRoot,
    edge_indices_to_id: &Vec<String>,
    preprocessed_tt: &HashMap<&String, HashMap<SerializedTimestamp, SerializedTravelTime>>,
) -> (Vec<EdgeId>, Vec<SerializedTravelTime>, Vec<SerializedTimestamp>) {
    // for edge and for every interval, write the travel time to the ipp files "first_ipp_of_arc", "ipp_travel_time", and "ipp_departure_time"
    // if in an interval the edge is not present, use the default travel time from edge_default_travel_times
    let mut first_ipp_of_arc: Vec<u32> = Vec::with_capacity(edge_indices_to_id.len() + 1);
    let mut ipp_travel_time = Vec::new();
    let mut ipp_departure_time = Vec::new();

    let mut added: u32 = 0;
    for edge_id in edge_indices_to_id.iter() {
        first_ipp_of_arc.push(added as u32);

        for interval in meandata.intervals.iter() {
            added += 1;
            let timestamp = (interval.begin * 1000.0) as SerializedTimestamp;
            ipp_departure_time.push(timestamp);
            let tt = *preprocessed_tt.get(edge_id).unwrap().get(&timestamp).unwrap();

            ipp_travel_time.push(tt);
        }
    }
    // add a dummy edge for the last interval
    first_ipp_of_arc.push(added);

    (first_ipp_of_arc, ipp_travel_time, ipp_departure_time)
}

// Tests for the extract_interpolation_points_from_meandata function

#[cfg(test)]
pub mod tests {
    use rust_road_router::datastr::graph::floating_time_dependent::TDGraph;

    use crate::sumo::meandata;

    #[test]
    fn test_extract_interpolation_points_from_meandata() {
        let edges: Vec<String> = vec!["edge1".to_string(), "edge2".to_string()];
        let edge_default_travel_times: Vec<u32> = vec![5_000, 3_000];

        let meandata = meandata::MeandataDocumentRoot {
            intervals: vec![
                meandata::Interval::create(
                    "interval1".to_string(),
                    0.0,
                    10.0,
                    vec![
                        meandata::Edge {
                            id: "edge1".to_string(),
                            traveltime: Some(4.0),
                            ..Default::default()
                        },
                        meandata::Edge {
                            id: "edge2".to_string(),
                            traveltime: None, // will use default travel time
                            ..Default::default()
                        },
                    ],
                ),
                meandata::Interval::create(
                    "interval2".to_string(),
                    10.0,
                    20.0,
                    vec![meandata::Edge {
                        id: "edge1".to_string(),
                        traveltime: Some(6.0),
                        ..Default::default()
                    }],
                ),
            ],
        };

        // should have 2 edges, each having 2 intervals. So we expect 4 interpolation points:
        // 1. edge1, interval1: 4.0
        // 2. edge2, interval1: 3.0 (default travel time)
        // 3. edge1, interval2: 6.0
        // 4. edge2, interval2: 3.0 (default travel time)
        let expected = (
            vec![0, 2, 4], // first_ipp_of_arc
            vec![
                4_000, // edge1, interval1
                6_000, // edge1, interval2
                3_000, // edge2, interval1 (default travel time)
                3_000, // edge2, interval2 (default travel time)
            ],
            vec![
                0_000,  // edge1, interval1
                10_000, // edge1, interval2
                0_000,  // edge2, interval1 (default travel time)
                10_000, // edge2, interval2 (default travel time)
            ],
        );

        let (first_ipp_of_arc, ipp_travel_time, ipp_departure_time) =
            super::extract_interpolation_points_from_meandata(&meandata, &edges, &edge_default_travel_times);

        assert_eq!(expected.0, first_ipp_of_arc);
        assert_eq!(expected.1, ipp_travel_time);
        assert_eq!(expected.2, ipp_departure_time);

        // can create new TDGraph
        // add first_out (vector where using node indices finds the first outgoing edge)
        // two edges, three nodes, having two outgoing edges each
        let first_out = vec![0, 1, 2];
        let head = vec![1, 2];

        TDGraph::new(first_out, head, first_ipp_of_arc, ipp_departure_time, ipp_travel_time);
    }

    #[test]
    fn test_handle_non_fifo_inputs() {
        let edges: Vec<String> = vec!["edge1".to_string(), "edge2".to_string()];
        let edge_default_travel_times: Vec<u32> = vec![5_000, 3_000];

        let meandata = meandata::MeandataDocumentRoot {
            intervals: vec![
                meandata::Interval::create(
                    "interval1".to_string(),
                    0.0,
                    10.0,
                    vec![
                        meandata::Edge {
                            id: "edge1".to_string(),
                            traveltime: Some(20.0),
                            ..Default::default()
                        },
                        meandata::Edge {
                            id: "edge2".to_string(),
                            traveltime: None, // will use default travel time
                            ..Default::default()
                        },
                    ],
                ),
                meandata::Interval::create(
                    "interval2".to_string(),
                    10.0,
                    20.0,
                    vec![meandata::Edge {
                        id: "edge1".to_string(),
                        traveltime: Some(6.0),
                        ..Default::default()
                    }],
                ),
            ],
        };

        // should have 2 edges, each having 2 intervals. So we expect 4 interpolation points:
        // 1. edge1, interval1: 16.0
        // 2. edge2, interval1: 3.0 (default travel time)
        // 3. edge1, interval2: 6.0
        // 4. edge2, interval2: 3.0 (default travel time)
        let expected = (
            vec![0, 2, 4], // first_ipp_of_arc
            vec![
                16_000, // edge1, interval1
                6_000,  // edge1, interval2
                3_000,  // edge2, interval1 (default travel time)
                3_000,  // edge2, interval2 (default travel time)
            ],
            vec![
                0_000,  // edge1, interval1
                10_000, // edge1, interval2
                0_000,  // edge2, interval1 (default travel time)
                10_000, // edge2, interval2 (default travel time)
            ],
        );

        let (first_ipp_of_arc, ipp_travel_time, ipp_departure_time) =
            super::extract_interpolation_points_from_meandata(&meandata, &edges, &edge_default_travel_times);

        assert_eq!(expected.0, first_ipp_of_arc);
        assert_eq!(expected.1, ipp_travel_time);
        assert_eq!(expected.2, ipp_departure_time);

        // can create new TDGraph
        // add first_out (vector where using node indices finds the first outgoing edge)
        // two edges, three nodes, having two outgoing edges each
        let first_out = vec![0, 1, 2];
        let head = vec![1, 2];

        TDGraph::new(first_out, head, first_ipp_of_arc, ipp_departure_time, ipp_travel_time);
    }

    #[test]
    fn test_fifo_ensured_backwards() {
        let edges: Vec<String> = vec!["edge1".to_string()];
        let edge_default_travel_times: Vec<u32> = vec![150_000];

        let meandata = meandata::MeandataDocumentRoot {
            intervals: vec![
                meandata::Interval::create(
                    "interval1".to_string(),
                    0.0,
                    100.0,
                    vec![meandata::Edge {
                        id: "edge1".to_string(),
                        traveltime: Some(1100.0),
                        ..Default::default()
                    }],
                ),
                meandata::Interval::create(
                    "interval2".to_string(),
                    100.0,
                    200.0,
                    vec![meandata::Edge {
                        id: "edge1".to_string(),
                        traveltime: Some(1000.0),
                        ..Default::default()
                    }],
                ),
                meandata::Interval::create(
                    "interval2".to_string(),
                    200.0,
                    250.0,
                    vec![meandata::Edge {
                        id: "edge1".to_string(),
                        traveltime: Some(150.0),
                        ..Default::default()
                    }],
                ),
            ],
        };

        let expected = (
            vec![0, 3], // first_ipp_of_arc
            vec![
                350_000, // edge1, travel_time1
                250_000, // edge1, travel_time2
                150_000, // edge1, travel_time3
            ],
            vec![
                0,       // edge1, interval1
                100_000, // edge1, interval2
                200_000, // edge1, interval2
            ],
        );

        let (first_ipp_of_arc, ipp_travel_time, ipp_departure_time) =
            super::extract_interpolation_points_from_meandata(&meandata, &edges, &edge_default_travel_times);

        assert_eq!(expected.0, first_ipp_of_arc);
        assert_eq!(expected.1, ipp_travel_time);
        assert_eq!(expected.2, ipp_departure_time);

        // can create new TDGraph
        // add first_out (vector where using node indices finds the first outgoing edge)
        // one edge, two nodes, having one outgoing edge
        let first_out = vec![0, 1];
        let head = vec![1];

        TDGraph::new(first_out, head, first_ipp_of_arc, ipp_departure_time, ipp_travel_time);
    }

    #[test]
    fn test_specific_weights_for_fifo() {
        let edges: Vec<String> = vec!["edge1".to_string()];
        let edge_default_travel_times: Vec<u32> = vec![86_400_000];

        let meandata = meandata::MeandataDocumentRoot {
            intervals: vec![
                meandata::Interval::create(
                    "interval1".to_string(),
                    86_340.0,
                    86_400.0,
                    vec![meandata::Edge {
                        id: "edge1".to_string(),
                        traveltime: Some(70_874.95),
                        ..Default::default()
                    }],
                ),
                meandata::Interval::create(
                    "interval2".to_string(),
                    86_400.0,
                    86_460.0,
                    vec![meandata::Edge {
                        id: "edge1".to_string(),
                        traveltime: Some(70_643.718),
                        ..Default::default()
                    }],
                ),
            ],
        };

        // should have 1 edge, each having 2 intervals. So we expect 4 interpolation points:
        // 1. edge1, interval1: 16.0
        // 2. edge1, interval2: 6.0
        let expected = (
            vec![0, 2], // first_ipp_of_arc
            vec![
                70_643_718 + 60_000, // edge1, interval1
                70_643_718,          // edge1, interval2
            ],
            vec![
                86_340_000, // edge1, interval1
                86_400_000, // edge1, interval2
            ],
        );

        let (first_ipp_of_arc, ipp_travel_time, ipp_departure_time) =
            super::extract_interpolation_points_from_meandata(&meandata, &edges, &edge_default_travel_times);

        assert_eq!(expected.0, first_ipp_of_arc);
        assert_eq!(expected.1, ipp_travel_time);
        assert_eq!(expected.2, ipp_departure_time);

        // can create new TDGraph
        // add first_out (vector where using node indices finds the first outgoing edge)
        // one edge, two nodes, having one outgoing edge
        let first_out = vec![0, 1];
        let head = vec![1];

        TDGraph::new(first_out, head, first_ipp_of_arc, ipp_departure_time, ipp_travel_time);
    }

    #[test]
    fn test_fifo_ensured_periodically() {
        let edges: Vec<String> = vec!["edge1".to_string()];
        let edge_default_travel_times: Vec<u32> = vec![25_000];

        let meandata = meandata::MeandataDocumentRoot {
            intervals: vec![
                meandata::Interval::create(
                    "interval1".to_string(),
                    0.0,
                    100.0,
                    vec![meandata::Edge {
                        id: "edge1".to_string(),
                        traveltime: Some(50.0),
                        ..Default::default()
                    }],
                ),
                meandata::Interval::create(
                    "interval2".to_string(),
                    100.0,
                    200.0,
                    vec![meandata::Edge {
                        id: "edge1".to_string(),
                        traveltime: Some(1000.0),
                        ..Default::default()
                    }],
                ),
                meandata::Interval::create(
                    "interval2".to_string(),
                    200.0,
                    250.0,
                    vec![meandata::Edge {
                        id: "edge1".to_string(),
                        traveltime: Some(200.0),
                        ..Default::default()
                    }],
                ),
            ],
        };

        let expected = (
            vec![0, 3], // first_ipp_of_arc
            vec![
                50_000,  // edge1, travel_time1
                175_000, // edge1, travel_time2
                75_000,  // edge1, travel_time3 since the default travel time is 25_000 and the interval is 50_000
            ],
            vec![
                0,       // edge1, interval1
                100_000, // edge1, interval2
                200_000, // edge1, interval2
            ],
        );

        let (first_ipp_of_arc, ipp_travel_time, ipp_departure_time) =
            super::extract_interpolation_points_from_meandata(&meandata, &edges, &edge_default_travel_times);

        assert_eq!(expected.0, first_ipp_of_arc);
        assert_eq!(expected.1, ipp_travel_time);
        assert_eq!(expected.2, ipp_departure_time);

        // can create new TDGraph
        // add first_out (vector where using node indices finds the first outgoing edge)
        // one edge, two nodes, having one outgoing edge
        let first_out = vec![0, 1];
        let head = vec![1];

        TDGraph::new(first_out, head, first_ipp_of_arc, ipp_departure_time, ipp_travel_time);
    }
}
