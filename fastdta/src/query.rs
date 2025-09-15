use std::path::Path;

use conversion::{
    FILE_QUERIES_DEPARTURE, FILE_QUERIES_FROM, FILE_QUERIES_TO, FILE_QUERY_ORIGINAL_FROM_EDGES, FILE_QUERY_ORIGINAL_TO_EDGES, SerializedTimestamp,
};

use rust_road_router::algo::catchup::Server;
use rust_road_router::algo::{self, TDQuery, TDQueryServer};
use rust_road_router::datastr::graph::EdgeId;
use rust_road_router::datastr::graph::floating_time_dependent::{FlWeight, TDGraph, Timestamp};
use rust_road_router::io::Load;

pub fn get_paths_with_cch_queries(query_server: &mut Server, input_dir: &Path, graph: &TDGraph) -> (Vec<Vec<EdgeId>>, Vec<FlWeight>, Vec<SerializedTimestamp>) {
    get_paths_from_queries(
        |from_edge, to_edge, from: u32, to: u32, departure: Timestamp, graph: &TDGraph| {
            let from_edge_tt = graph.get_travel_time_along_path(departure, &[from_edge]);
            let delayed_departure = departure + from_edge_tt;

            let result = query_server.td_query(TDQuery {
                from,
                to,
                departure: delayed_departure,
            });

            if let Some(mut result) = result.found() {
                let mut distance = from_edge_tt + result.distance();

                let edge_path = result.edge_path();

                let mut path = Vec::with_capacity(edge_path.len() + 2);
                path.push(from_edge);
                path.extend(edge_path.iter().map(|edge| edge.0));
                path.push(to_edge);

                distance += graph.get_travel_time_along_path(departure + distance, &[to_edge]);

                Some((path, distance))
            } else {
                None
            }
        },
        input_dir,
        graph,
    )
}

pub fn get_paths_with_dijkstra_queries(
    query_server: &mut algo::dijkstra::query::floating_td_dijkstra::Server,
    input_dir: &Path,
    graph: &TDGraph,
) -> (Vec<Vec<EdgeId>>, Vec<FlWeight>, Vec<SerializedTimestamp>) {
    get_paths_from_queries(
        |from_edge, to_edge, from: u32, to: u32, departure: Timestamp, graph: &TDGraph| {
            let from_edge_tt = graph.get_travel_time_along_path(departure, &[from_edge]);
            let delayed_departure = departure + from_edge_tt;

            let result = query_server.td_query(TDQuery {
                from,
                to,
                departure: delayed_departure,
            });

            if let Some(mut result) = result.found() {
                let mut distance = from_edge_tt + result.distance();

                let edge_path = result.edge_path();

                let mut path = Vec::with_capacity(edge_path.len() + 2);
                path.push(from_edge);
                path.extend(edge_path.iter().map(|edge| edge.0));
                path.push(to_edge);

                distance += graph.get_travel_time_along_path(departure + distance, &[to_edge]);

                Some((path, distance))
            } else {
                None
            }
        },
        input_dir,
        graph,
    )
}

fn get_paths_from_queries<F: FnMut(EdgeId, EdgeId, u32, u32, Timestamp, &TDGraph) -> Option<(Vec<EdgeId>, FlWeight)>>(
    mut path_collector: F,
    input_dir: &Path,
    graph: &TDGraph,
) -> (Vec<Vec<EdgeId>>, Vec<FlWeight>, Vec<SerializedTimestamp>) {
    // read queries from input_dir
    let queries_from = Vec::<u32>::load_from(input_dir.join(FILE_QUERIES_FROM)).unwrap();
    let queries_to = Vec::<u32>::load_from(input_dir.join(FILE_QUERIES_TO)).unwrap();
    let queries_departure = Vec::<SerializedTimestamp>::load_from(input_dir.join(FILE_QUERIES_DEPARTURE)).unwrap();
    let queries_original_from_edges = Vec::<u32>::load_from(&input_dir.join(FILE_QUERY_ORIGINAL_FROM_EDGES)).unwrap();
    let queries_original_to_edges = Vec::<u32>::load_from(input_dir.join(FILE_QUERY_ORIGINAL_TO_EDGES)).unwrap();

    assert!(queries_from.len() == queries_to.len());
    assert!(queries_from.len() == queries_departure.len());

    let mut paths: Vec<Vec<EdgeId>> = Vec::with_capacity(queries_from.len());
    let mut distances = Vec::with_capacity(queries_from.len());

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
        } else {
            println!("No path found from {} to {} at {dep:?}", queries_from[i], queries_to[i]);
        }
    }
    // distances is in seconds
    (paths, distances, queries_departure)
}
