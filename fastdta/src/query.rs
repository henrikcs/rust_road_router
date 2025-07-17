use std::path::Path;

use conversion::{FILE_QUERIES_DEPARTURE, FILE_QUERIES_FROM, FILE_QUERIES_TO, SerializedTimestamp};
use rust_road_router::algo::catchup::Server;
use rust_road_router::algo::customizable_contraction_hierarchy::CCH;
use rust_road_router::algo::{PathServer, TDQuery, TDQueryServer};
use rust_road_router::datastr::graph::EdgeId;
use rust_road_router::datastr::graph::floating_time_dependent::{CustomizedGraph, FlWeight, Timestamp};
use rust_road_router::io::Load;

pub fn get_paths_from_queries(cch: &CCH, customized_graph: &CustomizedGraph, input_dir: &Path) -> (Vec<Vec<EdgeId>>, Vec<FlWeight>, Vec<SerializedTimestamp>) {
    // read queries from input_dir
    let queries_from = Vec::<u32>::load_from(input_dir.join(FILE_QUERIES_FROM)).unwrap();
    let queries_to = Vec::<u32>::load_from(input_dir.join(FILE_QUERIES_TO)).unwrap();
    let queries_departure = Vec::<SerializedTimestamp>::load_from(input_dir.join(FILE_QUERIES_DEPARTURE)).unwrap();

    let mut query_server = Server::new(&cch, &customized_graph);

    assert!(queries_from.len() == queries_to.len());
    assert!(queries_from.len() == queries_departure.len());

    let mut paths: Vec<Vec<EdgeId>> = Vec::with_capacity(queries_from.len());
    let mut distances = Vec::with_capacity(queries_from.len());

    for i in 0..queries_from.len() {
        let dep = queries_departure[i];
        println!(
            "Find Earliest Arrival #{i} From: {}, To: {}, Departure: {dep:?}",
            queries_from[i], queries_to[i],
        );
        let result = query_server.td_query(TDQuery {
            from: queries_from[i] as u32,
            to: queries_to[i] as u32,
            departure: Timestamp::from_millis(queries_departure[i]),
        });
        if let Some(mut result) = result.found() {
            let ea = result.distance();
            let path = result.data().reconstruct_edge_path().iter().map(|edge| edge.0).collect();
            paths.push(path);
            distances.push(ea);

            println!(
                "From: {}, To: {}, Departure: {dep:?}, Earliest Arrival: {:?}",
                queries_from[i], queries_to[i], ea
            );
        } else {
            println!("No path found from {} to {} at {dep:?}", queries_from[i], queries_to[i]);
        }
    }

    // distances is in seconds
    (paths, distances, queries_departure)
}
