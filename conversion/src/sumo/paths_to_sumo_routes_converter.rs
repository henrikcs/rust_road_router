use std::path::Path;

use flate2::write;
use rust_road_router::{
    datastr::graph::{floating_time_dependent::Timestamp, EdgeId},
    io::read_strings_from_file,
};

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
    path_sets: &Vec<Vec<Vec<EdgeId>>>,
    costs: &Vec<Vec<f64>>,
    probabilities: &Vec<Vec<f64>>,
    choices: &Vec<usize>,
    departures: &Vec<SerializedTimestamp>,
    edge_indices_to_id: &Vec<String>,
    write_alternative_paths: bool,
) {
    // TODO: if memory consumption during this phase is too high, we can rewrite this section to have references to strings instead of the full string for each path
    // e.g. paths = Vec<Vec<&String>> and it references to each edge sumo-id for each query
    // edge_indices_to_id should then be the "database" of the edge sumo-ids
    let trip_ids: Vec<String> = read_strings_from_file(&input_dir.join(FILE_QUERY_IDS)).unwrap();
    let original_from_edges: Vec<String> = read_strings_from_file(&input_dir.join(FILE_QUERY_ORIGINAL_FROM_EDGES)).unwrap();
    let original_to_edges: Vec<String> = read_strings_from_file(&input_dir.join(FILE_QUERY_ORIGINAL_TO_EDGES)).unwrap();

    // transform path_sets from EdgeId to Sumo Ids (which are strings) using the edge_indices_to_id mapping
    let path_sets: Vec<Vec<String>> = transform_to_sumo_paths(&path_sets, &original_from_edges, &original_to_edges, edge_indices_to_id);

    // extract paths from alternative_lists from choices:
    let paths: Vec<&String> = get_chosen_paths_from_alternatives(&path_sets, &choices);

    let sumo_routes = convert_to_sumo_routes(paths, &trip_ids, departures);
    let sumo_alt_routes = convert_to_sumo_alt_routes(&path_sets, &trip_ids, costs, probabilities, choices, departures);

    let current_iteration_dir = input_dir.join(format!("{iteration:0>3}"));
    let route_file_prefix = format!("{input_prefix}_{iteration:0>3}");
    // write to file
    SumoRoutesWriter::write(&current_iteration_dir.join(format!("{route_file_prefix}{ROUTES}")), &sumo_routes).expect("Failed to write SUMO routes to file");

    if write_alternative_paths {
        SumoRoutesWriter::write(&current_iteration_dir.join(format!("{route_file_prefix}{ALT_ROUTES}")), &sumo_alt_routes)
            .expect("Failed to write SUMO alternative routes to file");
    }
}

/// prepares a datastructure which can be serialized into a *.rou.xml for SUMO
fn convert_to_sumo_routes(paths: Vec<&String>, trip_ids: &Vec<String>, departures: &Vec<SerializedTimestamp>) -> RoutesDocumentRoot {
    // create RoutesDocumentRoot
    let mut routes = RoutesDocumentRoot { vehicles: Vec::new() };

    for (i, &path) in paths.iter().enumerate() {
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

    routes
}

fn convert_to_sumo_alt_routes(
    path_sets: &Vec<Vec<String>>,
    trip_ids: &Vec<String>,
    costs: &Vec<Vec<f64>>,
    probabilities: &Vec<Vec<f64>>,
    choices: &Vec<usize>,
    departures: &Vec<SerializedTimestamp>,
) -> RoutesDocumentRoot {
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
            let cost = costs[i][j];
            let probability = probabilities[i][j];

            alternative_routes.push(Route {
                edges: path.clone(),
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

fn get_chosen_paths_from_alternatives<'a>(path_sets: &'a Vec<Vec<String>>, choices: &Vec<usize>) -> Vec<&'a String> {
    debug_assert_eq!(path_sets.len(), choices.len());
    // return the reference of the path corresponding to the choice for each query
    path_sets.iter().enumerate().map(|(i, paths)| &paths[choices[i]]).collect()
}

fn transform_to_sumo_paths(
    path_sets: &Vec<Vec<Vec<EdgeId>>>,
    original_from_edges: &Vec<String>,
    original_to_edges: &Vec<String>,
    edge_indices_to_id: &Vec<String>,
) -> Vec<Vec<String>> {
    path_sets
        .iter()
        .enumerate()
        .map(|(i, paths)| {
            paths
                .iter()
                .map(|path| get_edges_from_path(path, &original_from_edges[i], &original_to_edges[i], edge_indices_to_id))
                .collect::<Vec<String>>()
        })
        .collect::<Vec<Vec<String>>>()
}

fn get_edges_from_path(path: &Vec<EdgeId>, prefix: &String, suffix: &String, edge_indices_to_id: &Vec<String>) -> String {
    let edges = path
        .iter()
        .map(|&edge_index| edge_indices_to_id[edge_index as usize].clone())
        .collect::<Vec<_>>()
        .join(" ");

    // prefix path with original_from_edges[i]
    // and suffix it with original_to_edges[i]
    if path.is_empty() {
        format!("{} {}", prefix, suffix)
    } else {
        format!("{} {} {}", prefix, edges, suffix)
    }
}
