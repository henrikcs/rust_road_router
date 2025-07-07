use std::path::Path;

use rust_road_router::io::Store;

use crate::{
    sumo::{meandata::MeandataDocumentRoot, meandata_reader::SumoMeandataReader, XmlReader},
    FILE_FIRST_IPP_OF_ARC, FILE_IPP_DEPARTURE_TIME, FILE_IPP_TRAVEL_TIME,
};

pub fn set_new_graph_weights_from_meandata_file(
    path_to_graph_weights: &Path,
    path_to_sumo_meandata: &Path,
    edge_indices_to_id: &Vec<String>,
    edge_default_travel_times: &Vec<f64>,
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
    edge_default_travel_times: &Vec<f64>,
) -> (Vec<u32>, Vec<f32>, Vec<f32>) {
    // for edge and for every interval, write the travel time to the ipp files "first_ipp_of_arc", "ipp_travel_time", and "ipp_departure_time"
    // if in an interval the edge is not present, use the default travel time from edge_default_travel_times
    let mut first_ipp_of_arc = Vec::new();
    let mut ipp_travel_time = Vec::new();
    let mut ipp_departure_time = Vec::new();

    for (edge_index, edge_id) in edge_indices_to_id.iter().enumerate() {
        for interval in &meandata.intervals {
            if let Some(edge) = interval.edges.iter().find(|e| e.id == *edge_id) {
                // found the interval, use its travel time
                first_ipp_of_arc.push(edge_index as u32);
                ipp_travel_time.push(edge.traveltime as f32);
                ipp_departure_time.push(interval.begin as f32);
            } else {
                // use default travel time
                first_ipp_of_arc.push(edge_index as u32);
                ipp_travel_time.push(edge_default_travel_times[edge_index] as f32);
                ipp_departure_time.push(interval.begin as f32); // or some other default value
            }
        }
    }

    (first_ipp_of_arc, ipp_travel_time, ipp_departure_time)
}
