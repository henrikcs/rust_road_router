use std::path::Path;

use crate::sumo::{
    routes::{Route, RoutesDocumentRoot, Vehicle},
    routes_writer::SumoRoutesWriter,
    XmlWriter,
};

pub fn write_paths_as_sumo_routes(
    file: &Path,
    paths: &Vec<Vec<u32>>,
    edge_indices_to_id: &Vec<String>,
    trip_indices_to_id: &Vec<String>,
    departures: &Vec<f64>,
) {
    // create RoutesDocumentRoot
    let mut routes = RoutesDocumentRoot { vehicles: Vec::new() };

    for (i, path) in paths.iter().enumerate() {
        let edges = path
            .iter()
            .map(|&edge_index| edge_indices_to_id[edge_index as usize].clone())
            .collect::<Vec<_>>()
            .join(" ");
        let vehicle = Vehicle {
            id: trip_indices_to_id[i].clone(),
            depart: departures[i],
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

    // write to file
    SumoRoutesWriter::write(file, &routes).expect("Failed to write SUMO routes to file");
}
