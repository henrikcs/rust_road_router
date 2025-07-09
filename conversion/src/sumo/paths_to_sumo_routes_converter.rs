use std::path::Path;

use rust_road_router::{datastr::graph::floating_time_dependent::Timestamp, io::read_strings_from_file};

use crate::{
    sumo::{
        routes::{Route, RoutesDocumentRoot, Vehicle},
        routes_writer::SumoRoutesWriter,
        XmlWriter, ROUTES,
    },
    SerializedTimestamp, FILE_QUERY_IDS, FILE_QUERY_ORIGINAL_FROM_EDGES, FILE_QUERY_ORIGINAL_TO_EDGES,
};

/// only writes the .rou.xml file
pub fn write_paths_as_sumo_routes(
    input_dir: &Path,
    input_prefix: &String,
    iteration: u32,
    paths: &Vec<Vec<u32>>,
    departures: &Vec<SerializedTimestamp>,
    edge_indices_to_id: &Vec<String>,
) {
    let sumo_routes = convert_to_sumo_routes(&input_dir, &paths, &edge_indices_to_id, &departures);

    let current_iteration_dir = input_dir.join(format!("{iteration:0>3}"));
    // write to file
    SumoRoutesWriter::write(&current_iteration_dir.join(format!("{input_prefix}_{iteration:0>3}{ROUTES}")), &sumo_routes)
        .expect("Failed to write SUMO routes to file");
}

/// prepares a datastructure which can be serialized into a *.rou.xml for SUMO
fn convert_to_sumo_routes(dir: &Path, paths: &Vec<Vec<u32>>, edge_indices_to_id: &Vec<String>, departures: &Vec<SerializedTimestamp>) -> RoutesDocumentRoot {
    let trip_ids: Vec<String> = read_strings_from_file(&dir.join(FILE_QUERY_IDS)).unwrap();
    let original_from_edges: Vec<String> = read_strings_from_file(&dir.join(FILE_QUERY_ORIGINAL_FROM_EDGES)).unwrap();
    let original_to_edges: Vec<String> = read_strings_from_file(&dir.join(FILE_QUERY_ORIGINAL_TO_EDGES)).unwrap();

    // create RoutesDocumentRoot
    let mut routes = RoutesDocumentRoot { vehicles: Vec::new() };

    for (i, path) in paths.iter().enumerate() {
        let edges = path
            .iter()
            .map(|&edge_index| edge_indices_to_id[edge_index as usize].clone())
            .collect::<Vec<_>>()
            .join(" ");

        // prefix path with original_from_edges[i]
        // and suffix it with original_to_edges[i]
        let edges = if path.is_empty() {
            format!("{} {}", original_from_edges[i], original_to_edges[i])
        } else {
            format!("{} {} {}", original_from_edges[i], edges, original_to_edges[i])
        };

        let vehicle = Vehicle {
            id: trip_ids[i].clone(),
            depart: Timestamp::from_millis(departures[i]).into(),
            depart_lane: Some(String::from("best")),
            depart_pos: Some(String::from("random")),
            depart_speed: Some(String::from("max")),
            route: Some(Route {
                edges,
                cost: None,
                probability: None,
            }),
            route_distribution: None,
        };
        routes.vehicles.push(vehicle);
    }

    routes
}
