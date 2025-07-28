use std::{
    fs,
    path::{Path, PathBuf},
};

use rust_road_router::{
    datastr::graph::{floating_time_dependent::TDGraph, EdgeId},
    io::{Load, Reconstruct, Store},
};

use crate::{
    sumo::{meandata::MeandataDocumentRoot, meandata_reader::SumoMeandataReader, SumoTravelTime, XmlReader, MIN_TRAVEL_TIME},
    SerializedTimestamp, SerializedTravelTime, FILE_EDGE_DEFAULT_TRAVEL_TIMES, FILE_FIRST_IPP_OF_ARC, FILE_IPP_DEPARTURE_TIME, FILE_IPP_TRAVEL_TIME,
};

pub fn get_graph_with_travel_times_from_previous_iteration(input_dir: &Path, iteration: u32, edge_ids: &Vec<String>) -> TDGraph {
    // if iteration > 0, we load the previous iteration's travel times
    if iteration > 0 {
        let previous_iteration_dir = input_dir.join(format!("{:0>3}", iteration - 1));

        extract_travel_times_from_previous_iteration(&previous_iteration_dir, &input_dir, &edge_ids);
    }

    // TODO: instead of reconstructing the graph from a file, we could create it in memory
    TDGraph::reconstruct_from(&input_dir).expect("Failed to reconstruct the time-dependent graph")
}

pub fn extract_travel_times_from_previous_iteration(previous_iteration_dir: &Path, path_to_graph_weights: &Path, edge_indices_to_id: &Vec<String>) {
    // dump file starts with "dump_" and ends with ".xml"
    let dump_file = get_meandata_file(&previous_iteration_dir);

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
    // for edge and for every interval, write the travel time to the ipp files "first_ipp_of_arc", "ipp_travel_time", and "ipp_departure_time"
    // if in an interval the edge is not present, use the default travel time from edge_default_travel_times
    let mut first_ipp_of_arc: Vec<u32> = Vec::with_capacity(edge_indices_to_id.len() + 1);
    let mut ipp_travel_time = Vec::new();
    let mut ipp_departure_time = Vec::new();

    let mut added: u32 = 0;
    for (edge_index, edge_id) in edge_indices_to_id.iter().enumerate() {
        first_ipp_of_arc.push(added as u32);

        for interval in &meandata.intervals {
            added += 1;
            ipp_departure_time.push((interval.begin * 1000.0) as SerializedTimestamp); // or some other default value

            if let Some(edge) = interval.edges.iter().find(|e| e.id == *edge_id) {
                // found the interval, use its travel time
                if let Some(tt) = edge.traveltime {
                    // should be at least 1 millisecond
                    ipp_travel_time.push((SumoTravelTime::max(tt, MIN_TRAVEL_TIME) * 1000.0) as SerializedTravelTime);
                    continue; // continue to the next interval
                }
            }

            // use default travel time
            ipp_travel_time.push(edge_default_travel_times[edge_index]);
        }
    }
    // add a dummy edge for the last interval
    first_ipp_of_arc.push(added);

    (first_ipp_of_arc, ipp_travel_time, ipp_departure_time)
}

fn get_meandata_file(previous_iteration_dir: &Path) -> PathBuf {
    fs::read_dir(previous_iteration_dir)
        .unwrap()
        .find(|entry| {
            // check if entry is a file
            entry.is_ok()
                && entry.as_ref().unwrap().file_type().unwrap().is_file()
                && entry
                    .as_ref()
                    .unwrap()
                    .file_name()
                    .to_str()
                    .unwrap()
                    // check if file name starts with "dump_" and ends with ".xml"
                    .starts_with("dump_")
                && entry.as_ref().unwrap().file_name().to_str().unwrap().ends_with(".xml")
        })
        .map(|entry| entry.unwrap().path())
        .unwrap()
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
                meandata::Interval {
                    id: "interval1".to_string(),
                    begin: 0.0,
                    end: 10.0,
                    edges: vec![
                        meandata::Edge {
                            id: "edge1".to_string(),
                            traveltime: Some(4.0),
                        },
                        meandata::Edge {
                            id: "edge2".to_string(),
                            traveltime: None, // will use default travel time
                        },
                    ],
                },
                meandata::Interval {
                    id: "interval2".to_string(),
                    begin: 10.0,
                    end: 20.0,
                    edges: vec![meandata::Edge {
                        id: "edge1".to_string(),
                        traveltime: Some(6.0),
                    }],
                },
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
}
