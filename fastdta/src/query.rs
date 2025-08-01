use std::path::Path;

use conversion::{FILE_QUERIES_DEPARTURE, FILE_QUERIES_FROM, FILE_QUERIES_TO, SerializedTimestamp};

use rust_road_router::algo::catchup::Server;
use rust_road_router::algo::{self, TDQuery, TDQueryServer};
use rust_road_router::datastr::graph::EdgeId;
use rust_road_router::datastr::graph::floating_time_dependent::{FlWeight, Timestamp};
use rust_road_router::io::Load;

pub fn get_paths_with_cch_queries(query_server: &mut Server, input_dir: &Path) -> (Vec<Vec<EdgeId>>, Vec<FlWeight>, Vec<SerializedTimestamp>) {
    get_paths_from_queries(
        |from: u32, to: u32, departure: Timestamp| {
            let result = query_server.td_query(TDQuery { from, to, departure });

            if let Some(mut result) = result.found() {
                let ea = result.distance();

                let edge_path = result.edge_path();

                let path = edge_path.iter().map(|edge| edge.0).collect();
                Some((path, ea))
            } else {
                None
            }
        },
        input_dir,
    )
}

pub fn get_paths_with_dijkstra_queries(
    query_server: &mut algo::dijkstra::query::floating_td_dijkstra::Server,
    input_dir: &Path,
) -> (Vec<Vec<EdgeId>>, Vec<FlWeight>, Vec<SerializedTimestamp>) {
    get_paths_from_queries(
        |from: u32, to: u32, departure: Timestamp| {
            let result = query_server.td_query(TDQuery { from, to, departure });

            if let Some(mut result) = result.found() {
                let ea = result.distance();

                let edge_path = result.edge_path();

                let path = edge_path.iter().map(|edge| edge.0).collect();
                Some((path, ea))
            } else {
                None
            }
        },
        input_dir,
    )
}

fn get_paths_from_queries<F: FnMut(u32, u32, Timestamp) -> Option<(Vec<EdgeId>, FlWeight)>>(
    mut path_collector: F,
    input_dir: &Path,
) -> (Vec<Vec<EdgeId>>, Vec<FlWeight>, Vec<SerializedTimestamp>) {
    // read queries from input_dir
    let queries_from = Vec::<u32>::load_from(input_dir.join(FILE_QUERIES_FROM)).unwrap();
    let queries_to = Vec::<u32>::load_from(input_dir.join(FILE_QUERIES_TO)).unwrap();
    let queries_departure = Vec::<SerializedTimestamp>::load_from(input_dir.join(FILE_QUERIES_DEPARTURE)).unwrap();

    assert!(queries_from.len() == queries_to.len());
    assert!(queries_from.len() == queries_departure.len());

    let mut paths: Vec<Vec<EdgeId>> = Vec::with_capacity(queries_from.len());
    let mut distances = Vec::with_capacity(queries_from.len());

    for i in 0..queries_from.len() {
        let dep = queries_departure[i];

        if let Some((path, distance)) = path_collector(queries_from[i], queries_to[i], Timestamp::from_millis(dep)) {
            paths.push(path);
            distances.push(distance);
        } else {
            println!("No path found from {} to {} at {dep:?}", queries_from[i], queries_to[i]);
        }
    }
    // distances is in seconds
    (paths, distances, queries_departure)
}
