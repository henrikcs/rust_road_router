use std::path::Path;

use rust_road_router::{datastr::graph::floating_time_dependent::Timestamp, io::read_strings_from_file};

use crate::{
    sumo::{
        routes::{Route, RouteDistribution, RoutesDocumentRoot, Vehicle},
        routes_writer::SumoRoutesWriter,
        XmlWriter, ALT_ROUTES, ROUTES,
    },
    SerializedTimestamp, FILE_QUERY_IDS, FILE_QUERY_ORIGINAL_FROM_EDGES, FILE_QUERY_ORIGINAL_TO_EDGES,
};

/// only writes the .rou.xml file
pub fn write_paths_as_sumo_routes(
    input_dir: &Path,
    input_prefix: &String,
    iteration: u32,
    path_sets: &Vec<Vec<Vec<u32>>>,
    costs: &Vec<Vec<f64>>,
    probabilities: &Vec<Vec<f64>>,
    choices: &Vec<usize>,
    departures: &Vec<SerializedTimestamp>,
    edge_indices_to_id: &Vec<String>,
) {
    // extract paths from alternative_lists from choices:
    let paths: Vec<Vec<u32>> = get_chosen_paths_from_alternatives(path_sets, choices);

    let sumo_routes = convert_to_sumo_routes(&input_dir, &paths, &edge_indices_to_id, &departures);
    let sumo_alt_routes = convert_to_sumo_alt_routes(&input_dir, edge_indices_to_id, path_sets, costs, probabilities, choices, departures);

    let current_iteration_dir = input_dir.join(format!("{iteration:0>3}"));
    let route_file_prefix = format!("{input_prefix}_{iteration:0>3}");
    // write to file
    SumoRoutesWriter::write(&current_iteration_dir.join(format!("{route_file_prefix}{ROUTES}")), &sumo_routes).expect("Failed to write SUMO routes to file");

    SumoRoutesWriter::write(&current_iteration_dir.join(format!("{route_file_prefix}{ALT_ROUTES}")), &sumo_alt_routes)
        .expect("Failed to write SUMO alternative routes to file");
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
            depart_lane: None,
            depart_pos: None,
            depart_speed: None,
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

fn convert_to_sumo_alt_routes(
    dir: &Path,
    edge_indices_to_id: &Vec<String>,
    path_sets: &Vec<Vec<Vec<u32>>>,
    costs: &Vec<Vec<f64>>,
    probabilities: &Vec<Vec<f64>>,
    choices: &Vec<usize>,
    departures: &Vec<SerializedTimestamp>,
) -> RoutesDocumentRoot {
    let trip_ids: Vec<String> = read_strings_from_file(&dir.join(FILE_QUERY_IDS)).unwrap();

    debug_assert_eq!(trip_ids.len(), path_sets.len());
    debug_assert_eq!(trip_ids.len(), costs.len());
    debug_assert_eq!(trip_ids.len(), probabilities.len());
    debug_assert_eq!(trip_ids.len(), departures.len());

    // create RoutesDocumentRoot
    let mut routes = RoutesDocumentRoot { vehicles: Vec::new() };

    for (i, trip_id) in trip_ids.iter().enumerate() {
        // query i has the following alternatives
        let mut alternative_routes = vec![];
        for (j, path) in path_sets[i].iter().enumerate() {
            // create a route for each alternative path
            let edges = path
                .iter()
                .map(|&edge_index| edge_indices_to_id[edge_index as usize].clone())
                .collect::<Vec<_>>()
                .join(" ");

            let cost = costs[i][j];
            let probability = probabilities[i][j];

            alternative_routes.push(Route {
                edges,
                cost: Some(cost.into()),
                probability: Some(probability),
            });
        }
        let vehicle = Vehicle {
            id: trip_id.clone(),
            depart: Timestamp::from_millis(departures[i]).into(),
            depart_lane: None,
            depart_pos: None,
            depart_speed: None,
            route: None,
            route_distribution: Some(RouteDistribution {
                last: choices[i] as u32,
                routes: alternative_routes,
            }),
        };
        routes.vehicles.push(vehicle);
    }

    routes
}

fn get_chosen_paths_from_alternatives(path_sets: &Vec<Vec<Vec<u32>>>, choices: &Vec<usize>) -> Vec<Vec<u32>> {
    path_sets
        .iter()
        .enumerate()
        .map(|(i, alternatives)| {
            let choice_index = choices[i];
            if choice_index < alternatives.len() {
                alternatives[choice_index].clone()
            } else {
                vec![] // empty path if choice index is out of bounds
            }
        })
        .collect()
}
