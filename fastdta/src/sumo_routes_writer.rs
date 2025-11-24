use std::path::Path;

use conversion::{
    SerializedTimestamp,
    sumo::{
        FileWriter,
        routes::{Route, RoutesDocumentRoot, Vehicle},
        routes_writer::SumoRoutesWriter,
    },
};
use rust_road_router::datastr::graph::{EdgeId, floating_time_dependent::Timestamp};

/// Write paths as SUMO routes file for simulation input
/// This is similar to write_paths_as_sumo_routes but simplified for batch processing
pub fn write_batch_routes_for_sumo(
    output_path: &Path,
    trip_ids: &Vec<String>,
    paths: &Vec<Vec<EdgeId>>,
    departures: &Vec<SerializedTimestamp>,
    edge_indices_to_id: &Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Transform paths from EdgeId to SUMO edge IDs (strings)
    let sumo_paths: Vec<String> = paths
        .iter()
        .map(|path| {
            path.iter()
                .map(|&edge_index| edge_indices_to_id[edge_index as usize].clone())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();

    let sumo_routes = convert_to_sumo_routes(&sumo_paths, trip_ids, departures);

    SumoRoutesWriter::write(output_path, &sumo_routes)?;

    Ok(())
}

/// Prepares a data structure which can be serialized into a *.rou.xml for SUMO
fn convert_to_sumo_routes(paths: &Vec<String>, trip_ids: &Vec<String>, departures: &Vec<SerializedTimestamp>) -> RoutesDocumentRoot {
    let mut routes = RoutesDocumentRoot { vehicles: Vec::new() };

    for (i, path) in paths.iter().enumerate() {
        let vehicle = Vehicle {
            id: trip_ids[i].clone(),
            depart: Timestamp::from_millis(departures[i]).into(),
            depart_lane: None,
            depart_pos: None,
            depart_speed: None,
            route: Some(Route {
                edges: path.clone(),
                cost: None,
                probability: None,
            }),
            route_distribution: None,
        };
        routes.vehicles.push(vehicle);
    }

    // Sort by departure time
    routes.vehicles.sort_by(|a, b| ((a.depart * 1000.0) as u32).cmp(&((b.depart * 1000.0) as u32)));

    routes
}
