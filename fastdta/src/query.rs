use std::path::Path;

use conversion::{
    FILE_QUERIES_DEPARTURE, FILE_QUERIES_FROM, FILE_QUERIES_TO, FILE_QUERY_ORIGINAL_FROM_EDGES, FILE_QUERY_ORIGINAL_TO_EDGES, MIN_EDGE_WEIGHT,
    SerializedTimestamp,
};

use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use rust_road_router::algo::catchup::Server;
use rust_road_router::algo::customizable_contraction_hierarchy::CCH;
use rust_road_router::algo::{TDQuery, TDQueryServer};
use rust_road_router::datastr::graph::floating_time_dependent::{CustomizedGraph, FlWeight, TDGraph, Timestamp};
use rust_road_router::datastr::graph::{EdgeId, EdgeIdT};
use rust_road_router::io::Load;

pub fn get_paths_with_cch(
    cch: &CCH,
    customized_graph: &CustomizedGraph,
    input_dir: &Path,
    graph: &TDGraph,
) -> (Vec<Vec<EdgeId>>, Vec<FlWeight>, Vec<SerializedTimestamp>) {
    let (queries_from, queries_to, queries_departure, queries_original_from_edges, queries_original_to_edges) = read_queries(input_dir);
    get_paths_with_cch_queries(
        cch,
        customized_graph,
        &queries_from,
        &queries_to,
        &queries_departure,
        &queries_original_from_edges,
        &queries_original_to_edges,
        graph,
    )
}

pub fn get_paths_with_cch_queries(
    cch: &CCH,
    customized_graph: &CustomizedGraph,
    queries_from: &Vec<u32>,
    queries_to: &Vec<u32>,
    queries_departure: &Vec<SerializedTimestamp>,
    queries_original_from_edges: &Vec<u32>,
    queries_original_to_edges: &Vec<u32>,
    graph: &TDGraph,
) -> (Vec<Vec<EdgeId>>, Vec<FlWeight>, Vec<SerializedTimestamp>) {
    get_paths_from_queries_par(
        |from_edge, to_edge, from: u32, to: u32, departure: Timestamp, graph: &TDGraph| {
            let from_edge_tt = graph.get_travel_time_along_path(departure, &[from_edge]);

            if from_edge == to_edge {
                // special case: from and to are the same edge
                return Some((vec![from_edge], from_edge_tt));
            }

            let delayed_departure = departure + from_edge_tt;

            let mut server = Server::new(&cch, &customized_graph);
            let result = server.td_query(TDQuery {
                from,
                to,
                departure: delayed_departure,
            });

            if let Some(mut result) = result.found() {
                let edge_path = result.edge_path();

                let mut path = Vec::with_capacity(edge_path.len() + 2);
                path.push(from_edge);
                path.extend(edge_path.iter().map(|edge| edge.0));
                path.push(to_edge);

                let mut distance = from_edge_tt + result.distance();
                distance += graph.get_travel_time_along_path(departure + distance, &[to_edge]);

                Some((path, distance))
            } else {
                println!(
                    "No path found from {} to {} at {departure:?}",
                    queries_original_from_edges[from as usize], queries_original_to_edges[to as usize]
                );
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

pub fn get_paths_with_dijkstra(input_dir: &Path, graph: &TDGraph) -> (Vec<Vec<EdgeId>>, Vec<FlWeight>, Vec<SerializedTimestamp>) {
    let (queries_from, queries_to, queries_departure, queries_original_from_edges, queries_original_to_edges) = read_queries(input_dir);
    get_paths_with_dijkstra_queries(
        &queries_from,
        &queries_to,
        &queries_departure,
        &queries_original_from_edges,
        &queries_original_to_edges,
        graph,
    )
}
pub fn get_paths_with_dijkstra_queries(
    queries_from: &Vec<u32>,
    queries_to: &Vec<u32>,
    queries_departure: &Vec<SerializedTimestamp>,
    queries_original_from_edges: &Vec<u32>,
    queries_original_to_edges: &Vec<u32>,
    graph: &TDGraph,
) -> (Vec<Vec<EdgeId>>, Vec<FlWeight>, Vec<SerializedTimestamp>) {
    get_paths_from_queries_par(
        move |from_edge, to_edge, from: u32, to: u32, departure: Timestamp, graph: &TDGraph| {
            let from_edge_tt = graph.get_travel_time_along_path(departure, &[from_edge]);

            if from_edge == to_edge {
                // special case: from and to are the same edge
                return Some((vec![from_edge], from_edge_tt));
            }

            let delayed_departure = departure + from_edge_tt;

            let mut server = rust_road_router::algo::dijkstra::query::floating_td_dijkstra::Server::new(graph);
            let result = server.td_query(TDQuery {
                from,
                to,
                departure: delayed_departure,
            });

            if let Some(mut result) = result.found() {
                let edge_path = result.edge_path();

                let mut path = Vec::with_capacity(edge_path.len() + 2);
                path.push(from_edge);
                path.extend(edge_path.iter().map(|edge| edge.0));
                path.push(to_edge);

                let mut distance = from_edge_tt + result.distance();
                distance += graph.get_travel_time_along_path(departure + distance, &[to_edge]);

                Some((path, distance))
            } else {
                println!(
                    "No path found from {} to {} at {departure:?}",
                    queries_original_from_edges[from as usize], queries_original_to_edges[to as usize]
                );
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

fn get_paths_from_queries_par<
    F: FnMut(EdgeId, EdgeId, u32, u32, Timestamp, &TDGraph) -> Option<(Vec<EdgeId>, FlWeight)> + std::marker::Sync + std::marker::Send + Clone,
>(
    path_collector: F,
    queries_from: &Vec<u32>,
    queries_to: &Vec<u32>,
    queries_departure: &Vec<SerializedTimestamp>,
    queries_original_from_edges: &Vec<u32>,
    queries_original_to_edges: &Vec<u32>,
    graph: &TDGraph,
) -> (Vec<Vec<EdgeId>>, Vec<FlWeight>, Vec<SerializedTimestamp>) {
    let mut pdds: Vec<(Vec<EdgeId>, FlWeight, SerializedTimestamp)> = vec![(vec![], FlWeight::INFINITY, 0); queries_from.len()];

    pdds.par_iter_mut()
        .enumerate()
        .for_each_with(path_collector, |path_collector, (i, (path, tt, dep))| {
            let departure = queries_departure[i];
            departure.clone_into(dep);

            if let Some((shortest_path, shortest_travel_time)) = path_collector(
                queries_original_from_edges[i],
                queries_original_to_edges[i],
                queries_from[i],
                queries_to[i],
                Timestamp::from_millis(departure),
                &graph,
            ) {
                shortest_path.clone_into(path);
                shortest_travel_time.clone_into(tt);
            } else {
                println!(
                    "No path found from {} to {} at {dep:?} in query {}",
                    queries_original_from_edges[i], queries_original_to_edges[i], i
                );
            }
        });

    let mut paths = Vec::with_capacity(queries_from.len());
    let mut distances = Vec::with_capacity(queries_from.len());
    let mut departures = Vec::with_capacity(queries_from.len());

    pdds.into_iter().for_each(|(path, distance, departure)| {
        paths.push(path);
        distances.push(distance);
        departures.push(departure);
    });

    (paths, distances, departures)
}

fn _get_paths_from_queries<F: FnMut(EdgeId, EdgeId, u32, u32, Timestamp, &TDGraph) -> Option<(Vec<EdgeId>, FlWeight)>>(
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

fn _construct_path_and_time(
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

    // remaining_path starts and ends with connection edges, which we do not want to count towards the total distance
    distance = distance - FlWeight::new(MIN_EDGE_WEIGHT * ((remaining_path.len() + 1) / 2) as f64); // subtract connection edge weights

    (path, distance)
}
