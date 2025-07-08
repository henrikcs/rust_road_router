use std::path::Path;

use rust_road_router::io::read_strings_from_file;

use crate::{
    sumo::{
        routes::{Route, RoutesDocumentRoot, Vehicle},
        routes_reader::SumoRoutesReader,
        routes_writer::SumoRoutesWriter,
        XmlReader, XmlWriter, ROUTES,
    },
    SerializedTimestamp, FILE_QUERY_IDS, FILE_QUERY_ORIGINAL_FROM_EDGES, FILE_QUERY_ORIGINAL_TO_EDGES,
};

pub fn write_paths_as_sumo_routes(
    input_dir: &Path,
    input_prefix: &String,
    iteration: u32,
    paths: &Vec<Vec<u32>>,
    departures: &Vec<SerializedTimestamp>,
    edge_indices_to_id: &Vec<String>,
) {
    let trip_ids: Vec<String> = read_strings_from_file(&input_dir.join(FILE_QUERY_IDS)).unwrap();
    let original_from_edges: Vec<String> = read_strings_from_file(&input_dir.join(FILE_QUERY_ORIGINAL_FROM_EDGES)).unwrap();
    let original_to_edges: Vec<String> = read_strings_from_file(&input_dir.join(FILE_QUERY_ORIGINAL_TO_EDGES)).unwrap();

    let sumo_routes = convert_to_sumo_routes(
        None,
        &paths,
        &edge_indices_to_id,
        &trip_ids,
        &original_from_edges,
        &original_to_edges,
        &departures,
    );

    let current_iteration_dir = input_dir.join(format!("{iteration:0>3}"));
    // write to file
    SumoRoutesWriter::write(&current_iteration_dir.join(format!("{input_prefix}_{iteration:0>3}{ROUTES}")), &sumo_routes)
        .expect("Failed to write SUMO routes to file");
}

/// prepares an rou.alt.xml file and rou.xml for SUMO
fn convert_to_sumo_routes(
    previous_iteration_dir: Option<&Path>,
    paths: &Vec<Vec<u32>>,
    edge_indices_to_id: &Vec<String>,
    trip_indices_to_id: &Vec<String>,
    original_from_edges: &Vec<String>,
    original_to_edges: &Vec<String>,
    departures: &Vec<SerializedTimestamp>,
) -> RoutesDocumentRoot {
    // create RoutesDocumentRoot
    let mut routes = RoutesDocumentRoot { vehicles: Vec::new() };

    if let Some(previous_iteration_dir) = previous_iteration_dir {
        //TODO: load alternative routes from previous iteration
        let _alternative_routes = SumoRoutesReader::read(&previous_iteration_dir.join("rou.alt.xml"));
    }

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
            id: trip_indices_to_id[i].clone(),
            depart: departures[i].into(),
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
