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
    original_from_edges: &Vec<String>,
    original_to_edges: &Vec<String>,
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

        // prefix path with original_from_edges[i]
        // and suffix it with original_to_edges[i]
        let edges = if path.is_empty() {
            format!("{} {}", original_from_edges[i], original_to_edges[i])
        } else {
            format!("{} {} {}", original_from_edges[i], edges, original_to_edges[i])
        };

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
