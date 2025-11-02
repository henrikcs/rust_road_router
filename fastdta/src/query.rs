use std::path::Path;

use conversion::{
    FILE_QUERIES_DEPARTURE, FILE_QUERIES_FROM, FILE_QUERIES_TO, FILE_QUERY_ORIGINAL_FROM_EDGES, FILE_QUERY_ORIGINAL_TO_EDGES, SerializedTimestamp,
};

use rust_road_router::algo::catchup::Server;
use rust_road_router::algo::{self, TDQuery, TDQueryServer};
use rust_road_router::datastr::graph::floating_time_dependent::{FlWeight, TDGraph, Timestamp};
use rust_road_router::datastr::graph::{EdgeId, EdgeIdT};
use rust_road_router::io::Load;

pub fn get_paths_with_cch(query_server: &mut Server, input_dir: &Path, graph: &TDGraph) -> (Vec<Vec<EdgeId>>, Vec<FlWeight>, Vec<SerializedTimestamp>) {
    let (queries_from, queries_to, queries_departure, queries_original_from_edges, queries_original_to_edges) = read_queries(input_dir);
    get_paths_with_cch_queries(
        query_server,
        &queries_from,
        &queries_to,
        &queries_departure,
        &queries_original_from_edges,
        &queries_original_to_edges,
        graph,
    )
}

pub fn get_paths_with_cch_queries(
    query_server: &mut Server,
    queries_from: &Vec<u32>,
    queries_to: &Vec<u32>,
    queries_departure: &Vec<SerializedTimestamp>,
    queries_original_from_edges: &Vec<u32>,
    queries_original_to_edges: &Vec<u32>,
    graph: &TDGraph,
) -> (Vec<Vec<EdgeId>>, Vec<FlWeight>, Vec<SerializedTimestamp>) {
    get_paths_from_queries(
        |from_edge, to_edge, from: u32, to: u32, departure: Timestamp, graph: &TDGraph| {
            let from_edge_tt = graph.get_travel_time_along_path(departure, &[from_edge]);

            if from_edge == to_edge {
                // special case: from and to are the same edge
                return Some((vec![from_edge], from_edge_tt));
            }

            let delayed_departure = departure + from_edge_tt;

            let result = query_server.td_query(TDQuery {
                from,
                to,
                departure: delayed_departure,
            });

            if let Some(mut result) = result.found() {
                let (path, distance) = construct_path_and_time(graph, from_edge, from_edge_tt, to_edge, departure, result.edge_path(), result.distance());

                Some((path, distance))
            } else {
                None
            }
        },
        queries_from,
        queries_to,
        queries_departure,
        queries_original_from_edges,
        queries_original_to_edges,
        graph,
    )
}

pub fn get_paths_with_dijkstra(
    query_server: &mut algo::dijkstra::query::floating_td_dijkstra::Server,
    input_dir: &Path,
    graph: &TDGraph,
) -> (Vec<Vec<EdgeId>>, Vec<FlWeight>, Vec<SerializedTimestamp>) {
    let (queries_from, queries_to, queries_departure, queries_original_from_edges, queries_original_to_edges) = read_queries(input_dir);
    get_paths_with_dijkstra_queries(
        query_server,
        &queries_from,
        &queries_to,
        &queries_departure,
        &queries_original_from_edges,
        &queries_original_to_edges,
        graph,
    )
}

pub fn get_paths_with_dijkstra_queries(
    query_server: &mut algo::dijkstra::query::floating_td_dijkstra::Server,
    queries_from: &Vec<u32>,
    queries_to: &Vec<u32>,
    queries_departure: &Vec<SerializedTimestamp>,
    queries_original_from_edges: &Vec<u32>,
    queries_original_to_edges: &Vec<u32>,
    graph: &TDGraph,
) -> (Vec<Vec<EdgeId>>, Vec<FlWeight>, Vec<SerializedTimestamp>) {
    get_paths_from_queries(
        |from_edge, to_edge, from: u32, to: u32, departure: Timestamp, graph: &TDGraph| {
            let from_edge_tt = graph.get_travel_time_along_path(departure, &[from_edge]);

            if from_edge == to_edge {
                // special case: from and to are the same edge
                return Some((vec![from_edge], from_edge_tt));
            }

            let delayed_departure = departure + from_edge_tt;

            let result = query_server.td_query(TDQuery {
                from,
                to,
                departure: delayed_departure,
            });

            if let Some(mut result) = result.found() {
                let (path, distance) = construct_path_and_time(graph, from_edge, from_edge_tt, to_edge, departure, result.edge_path(), result.distance());

                Some((path, distance))
            } else {
                None
            }
        },
        queries_from,
        queries_to,
        queries_departure,
        queries_original_from_edges,
        queries_original_to_edges,
        graph,
    )
}

pub fn read_queries(input_dir: &Path) -> (Vec<u32>, Vec<u32>, Vec<SerializedTimestamp>, Vec<u32>, Vec<u32>) {
    let queries_from = Vec::<u32>::load_from(input_dir.join(FILE_QUERIES_FROM)).unwrap();
    let queries_to = Vec::<u32>::load_from(input_dir.join(FILE_QUERIES_TO)).unwrap();
    let queries_departure: Vec<SerializedTimestamp> = Vec::<SerializedTimestamp>::load_from(input_dir.join(FILE_QUERIES_DEPARTURE)).unwrap();
    let queries_original_from_edges = Vec::<u32>::load_from(&input_dir.join(FILE_QUERY_ORIGINAL_FROM_EDGES)).unwrap();
    let queries_original_to_edges = Vec::<u32>::load_from(input_dir.join(FILE_QUERY_ORIGINAL_TO_EDGES)).unwrap();

    assert!(queries_from.len() == queries_to.len());
    assert!(queries_from.len() == queries_departure.len());

    (
        queries_from,
        queries_to,
        queries_departure,
        queries_original_from_edges,
        queries_original_to_edges,
    )
}

fn get_paths_from_queries<F: FnMut(EdgeId, EdgeId, u32, u32, Timestamp, &TDGraph) -> Option<(Vec<EdgeId>, FlWeight)>>(
    mut path_collector: F,
    queries_from: &Vec<u32>,
    queries_to: &Vec<u32>,
    queries_departure: &Vec<SerializedTimestamp>,
    queries_original_from_edges: &Vec<u32>,
    queries_original_to_edges: &Vec<u32>,
    graph: &TDGraph,
) -> (Vec<Vec<EdgeId>>, Vec<FlWeight>, Vec<SerializedTimestamp>) {
    let mut paths: Vec<Vec<EdgeId>> = Vec::with_capacity(queries_from.len());
    let mut distances = Vec::with_capacity(queries_from.len());
    let mut departures = Vec::with_capacity(queries_from.len());

    for i in 0..queries_from.len() {
        let dep = queries_departure[i];

        if let Some((path, distance)) = path_collector(
            queries_original_from_edges[i],
            queries_original_to_edges[i],
            queries_from[i],
            queries_to[i],
            Timestamp::from_millis(dep),
            &graph,
        ) {
            paths.push(path);
            distances.push(distance);
            departures.push(dep);
        } else {
            println!(
                "No path found from {} to {} at {dep:?} in query {}",
                queries_original_from_edges[i], queries_original_to_edges[i], i
            );
            paths.push(vec![]);
            distances.push(FlWeight::INFINITY);
            departures.push(dep);
        }
    }
    // distances is in seconds
    (paths, distances, departures)
}

fn construct_path_and_time(
    graph: &TDGraph,
    from_edge: EdgeId,
    from_edge_tt: FlWeight,
    to_edge: EdgeId,
    departure: Timestamp,
    remaining_path: Vec<EdgeIdT>,
    remaining_path_distance: FlWeight,
) -> (Vec<EdgeId>, FlWeight) {
    let mut path = Vec::with_capacity(remaining_path.len() + 2);
    path.push(from_edge);

    // the edge_path alternates between internal edges and normal edges, the first edge being internal
    // we only want the normal edges, so we skip every second edge

    path.extend(remaining_path.iter().skip(1).step_by(2).map(|edge| edge.0));

    // path.extend(edge_path.iter().map(|edge| edge.0));
    path.push(to_edge);

    let mut distance = from_edge_tt + remaining_path_distance;
    distance += graph.get_travel_time_along_path(departure + distance, &[to_edge]);

    (path, distance)
}
