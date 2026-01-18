use std::path::Path;

use conversion::{
    FILE_QUERIES_DEPARTURE, FILE_QUERIES_FROM, FILE_QUERIES_TO, FILE_QUERY_ORIGINAL_FROM_EDGES, FILE_QUERY_ORIGINAL_TO_EDGES, SerializedTimestamp,
};

#[cfg(feature = "expand-sumo-nodes")]
use conversion::MIN_EDGE_WEIGHT;

#[cfg(not(feature = "queries-disable-par"))]
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
#[cfg(not(feature = "queries-disable-par"))]
use rust_road_router::algo::{catchup::floating_td_stepped_elimination_tree::FloatingTDSteppedEliminationTree, customizable_contraction_hierarchy::CCHT};

use rust_road_router::algo::catchup::Server;
use rust_road_router::algo::customizable_contraction_hierarchy::CCH;

use rust_road_router::algo::dijkstra::query::floating_td_dijkstra;
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
    #[cfg(feature = "queries-disable-par")]
    {
        let mut server = Server::new(&cch, &customized_graph);
        get_paths_from_queries(
            |from_edge, to_edge, from: u32, to: u32, departure: Timestamp, graph: &TDGraph| {
                let from_edge_tt = graph.get_travel_time_along_path(departure, &[from_edge]);

                if from_edge == to_edge {
                    // special case: from and to are the same edge
                    return Some((vec![from_edge], from_edge_tt));
                }

                let delayed_departure = departure + from_edge_tt;

                let result = server.td_query(TDQuery {
                    from,
                    to,
                    departure: delayed_departure,
                });

                if let Some(mut result) = result.found() {
                    let edge_path = result.edge_path();

                    let (path, distance) = construct_path_and_time(graph, from_edge, from_edge_tt, to_edge, departure, edge_path, result.distance());

                    Some((path, distance))
                } else {
                    println!("No path found from {} to {} at {departure:?}", from_edge, to_edge);
                    fallback(graph, from_edge, to_edge, from, to, departure, from_edge_tt, delayed_departure)
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

    #[cfg(not(feature = "queries-disable-par"))]
    {
        // Create template elimination trees once
        let forward_template = FloatingTDSteppedEliminationTree::new(customized_graph.upward_bounds_graph(), cch.elimination_tree());
        let backward_template = FloatingTDSteppedEliminationTree::new(customized_graph.downward_bounds_graph(), cch.elimination_tree());

        get_paths_from_queries_par(
            || {
                // Clone the elimination trees for each thread (cheaper than creating from scratch)
                Server::new_with_elimination_trees(&cch, &customized_graph, forward_template.clone(), backward_template.clone())
            },
            |server: &mut Server, from_edge, to_edge, from: u32, to: u32, departure: Timestamp, graph: &TDGraph| {
                let from_edge_tt = graph.get_travel_time_along_path(departure, &[from_edge]);

                if from_edge == to_edge {
                    // special case: from and to are the same edge
                    return Some((vec![from_edge], from_edge_tt));
                }

                let delayed_departure = departure + from_edge_tt;

                let result = server.td_query(TDQuery {
                    from,
                    to,
                    departure: delayed_departure,
                });

                if let Some(mut result) = result.found() {
                    let edge_path = result.edge_path();

                    let (path, distance) = construct_path_and_time(graph, from_edge, from_edge_tt, to_edge, departure, edge_path, result.distance());

                    Some((path, distance))
                } else {
                    println!("No path found from {} to {} at {departure:?}", from_edge, to_edge);

                    // there might be some cases where no path is found due to IPP issues
                    fallback(graph, from_edge, to_edge, from, to, departure, from_edge_tt, delayed_departure)
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
    #[cfg(feature = "queries-disable-par")]
    {
        let mut server = floating_td_dijkstra::Server::new(graph);

        get_paths_from_queries(
            move |from_edge, to_edge, from: u32, to: u32, departure: Timestamp, graph: &TDGraph| {
                let from_edge_tt = graph.get_travel_time_along_path(departure, &[from_edge]);

                if from_edge == to_edge {
                    // special case: from and to are the same edge
                    return Some((vec![from_edge], from_edge_tt));
                }

                let delayed_departure = departure + from_edge_tt;

                let result = server.td_query(TDQuery {
                    from,
                    to,
                    departure: delayed_departure,
                });

                if let Some(mut result) = result.found() {
                    let edge_path = result.edge_path();

                    let (path, distance) = construct_path_and_time(graph, from_edge, from_edge_tt, to_edge, departure, edge_path, result.distance());

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

    #[cfg(not(feature = "queries-disable-par"))]
    {
        get_paths_from_queries_par(
            || floating_td_dijkstra::Server::new(graph),
            |server: &mut floating_td_dijkstra::Server, from_edge, to_edge, from: u32, to: u32, departure: Timestamp, graph: &TDGraph| {
                let from_edge_tt = graph.get_travel_time_along_path(departure, &[from_edge]);

                if from_edge == to_edge {
                    // special case: from and to are the same edge
                    return Some((vec![from_edge], from_edge_tt));
                }

                let delayed_departure = departure + from_edge_tt;

                let result = server.td_query(TDQuery {
                    from,
                    to,
                    departure: delayed_departure,
                });

                if let Some(mut result) = result.found() {
                    let edge_path = result.edge_path();

                    let (path, distance) = construct_path_and_time(graph, from_edge, from_edge_tt, to_edge, departure, edge_path, result.distance());

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

fn fallback(
    graph: &TDGraph,
    from_edge: EdgeId,
    to_edge: EdgeId,
    from: u32,
    to: u32,
    departure: Timestamp,
    from_edge_tt: FlWeight,
    delayed_departure: Timestamp,
) -> Option<(Vec<EdgeId>, FlWeight)> {
    // As a fallback, use dijkstra routing
    let mut server = floating_td_dijkstra::Server::new(graph);

    let result = server.td_query(TDQuery {
        from,
        to,
        departure: delayed_departure,
    });

    if let Some(mut result) = result.found() {
        let edge_path = result.edge_path();

        let (path, distance) = construct_path_and_time(graph, from_edge, from_edge_tt, to_edge, departure, edge_path, result.distance());

        Some((path, distance))
    } else {
        println!("Dijkstra fallback also found no path from {} to {} at {departure:?}", from_edge, to_edge);
        None
    }
}

#[cfg(not(feature = "queries-disable-par"))]
fn get_paths_from_queries_par<
    ServerT,
    InitF: Fn() -> ServerT + std::marker::Sync + std::marker::Send,
    F: Fn(&mut ServerT, EdgeId, EdgeId, u32, u32, Timestamp, &TDGraph) -> Option<(Vec<EdgeId>, FlWeight)> + std::marker::Sync + std::marker::Send,
>(
    server_init: InitF,
    path_collector: F,
    queries_from: &Vec<u32>,
    queries_to: &Vec<u32>,
    queries_departure: &Vec<SerializedTimestamp>,
    queries_original_from_edges: &Vec<u32>,
    queries_original_to_edges: &Vec<u32>,
    graph: &TDGraph,
) -> (Vec<Vec<EdgeId>>, Vec<FlWeight>, Vec<SerializedTimestamp>) {
    use rayon::iter::IntoParallelRefIterator;

    let num_queries = queries_from.len();
    let num_threads = rayon::current_num_threads();
    let num_chunks = num_threads.min(num_queries);

    // Calculate chunk size, ensuring all queries are covered
    let base_chunk_size = num_queries / num_chunks;
    let remainder = num_queries % num_chunks;

    // Create chunk ranges
    let mut chunk_ranges = Vec::with_capacity(num_chunks);
    let mut start = 0;
    for i in 0..num_chunks {
        // Distribute remainder across first chunks
        let chunk_size = base_chunk_size + if i < remainder { 1 } else { 0 };
        let end = start + chunk_size;
        chunk_ranges.push(start..end);
        start = end;
    }

    // Process chunks in parallel, but queries within each chunk sequentially
    // Each thread returns a hashmap with query_id -> (path, distance, departure)
    let chunk_results: Vec<std::collections::HashMap<usize, (Vec<EdgeId>, FlWeight, SerializedTimestamp)>> = chunk_ranges
        .par_iter()
        .map_init(
            || server_init(),
            |server, range| {
                let mut local_results = std::collections::HashMap::new();

                for i in range.clone() {
                    let departure = queries_departure[i];

                    if let Some((shortest_path, shortest_travel_time)) = path_collector(
                        server,
                        queries_original_from_edges[i],
                        queries_original_to_edges[i],
                        queries_from[i],
                        queries_to[i],
                        Timestamp::from_millis(departure),
                        &graph,
                    ) {
                        local_results.insert(i, (shortest_path, shortest_travel_time, departure));
                    } else {
                        println!(
                            "No path found from {} to {} at {departure:?} in query {}",
                            queries_original_from_edges[i], queries_original_to_edges[i], i
                        );
                    }
                }

                local_results
            },
        )
        .collect();

    // Merge results from all threads into ordered vectors
    let mut paths = Vec::with_capacity(num_queries);
    let mut distances = Vec::with_capacity(num_queries);
    let mut departures = Vec::with_capacity(num_queries);

    for i in 0..num_queries {
        // Find the result for query i in the chunk results
        let mut found = false;
        for chunk_result in &chunk_results {
            if let Some((path, distance, departure)) = chunk_result.get(&i) {
                paths.push(path.clone());
                distances.push(*distance);
                departures.push(*departure);
                found = true;
                break;
            }
        }

        if !found {
            // Query failed, insert empty result
            paths.push(vec![]);
            distances.push(FlWeight::INFINITY);
            departures.push(queries_departure[i]);
        }
    }

    (paths, distances, departures)
}

#[cfg(feature = "queries-disable-par")]
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
    _from_edge_tt: FlWeight,
    to_edge: EdgeId,
    departure: Timestamp,
    remaining_path: Vec<EdgeIdT>,
    _remaining_path_tt: FlWeight,
) -> (Vec<EdgeId>, FlWeight) {
    let mut path = Vec::with_capacity(remaining_path.len() + 2);
    path.push(from_edge);
    let mut distance;
    #[cfg(feature = "expand-sumo-nodes")]
    {
        // With node expansion: the edge_path alternates between internal edges and normal edges
        // The first edge is an internal connection edge, so we skip it and take every second edge
        path.extend(remaining_path.iter().skip(1).step_by(2).map(|edge| edge.0));
        path.push(to_edge);

        distance = graph.get_travel_time_along_path(departure, &path);
    }

    #[cfg(not(feature = "expand-sumo-nodes"))]
    {
        // Without node expansion: all edges in the path are normal edges
        path.extend(remaining_path.iter().map(|edge| edge.0));

        path.push(to_edge);
        distance = _from_edge_tt + _remaining_path_tt;
        distance += graph.get_travel_time_along_path(departure + distance, &[to_edge]);
    }

    #[cfg(feature = "expand-sumo-nodes")]
    {
        // remaining_path starts and ends with connection edges, which we do not want to count towards the total distance
        distance = distance - FlWeight::new(MIN_EDGE_WEIGHT * ((remaining_path.len() + 1) / 2) as f64);
    }

    (path, distance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use conversion::sumo::{
        edges::{Edge, EdgesDocumentRoot},
        nodes::NodesDocumentRoot,
    };
    use rust_road_router::datastr::graph::{Graph, floating_time_dependent::TDGraph};

    fn create_simple_test_graph() -> (NodesDocumentRoot, EdgesDocumentRoot, TDGraph) {
        let nodes = NodesDocumentRoot {
            nodes: vec![
                conversion::sumo::nodes::Node {
                    id: String::from("n1"),
                    x: 0.0,
                    y: 0.0,
                },
                conversion::sumo::nodes::Node {
                    id: String::from("n2"),
                    x: 1.0,
                    y: 0.0,
                },
                conversion::sumo::nodes::Node {
                    id: String::from("n3"),
                    x: 2.0,
                    y: 0.0,
                },
            ],
            location: None,
        };

        let edges = EdgesDocumentRoot {
            edges: vec![
                Edge {
                    id: String::from("e1"),
                    from: String::from("n1"),
                    to: String::from("n2"),
                    num_lanes: Some(1),
                    speed: Some(10.0),
                    length: Some(100.0),
                    lanes: vec![],
                    params: vec![],
                    priority: Some(-1),
                },
                Edge {
                    id: String::from("e2"),
                    from: String::from("n2"),
                    to: String::from("n3"),
                    num_lanes: Some(1),
                    speed: Some(10.0),
                    length: Some(100.0),
                    lanes: vec![],
                    params: vec![],
                    priority: Some(-1),
                },
            ],
        };

        #[cfg(feature = "expand-sumo-nodes")]
        let connections = conversion::sumo::connections::ConnectionsDocumentRoot {
            connections: vec![conversion::sumo::connections::Connection {
                from: String::from("e1"),
                to: String::from("e2"),
                from_lane: Some(String::from("0")),
                to_lane: Some(String::from("0")),
            }],
        };

        #[cfg(feature = "expand-sumo-nodes")]
        let (routing_kit_graph, _, _, _) =
            conversion::sumo::sumo_to_td_graph_converter::get_routing_kit_td_graph_from_sumo(&nodes, &edges, &connections, None, None, None);

        #[cfg(not(feature = "expand-sumo-nodes"))]
        let (routing_kit_graph, _, _) = conversion::sumo::sumo_to_td_graph_converter::get_routing_kit_td_graph_from_sumo(&nodes, &edges, None, None, None);

        let graph = TDGraph::new(
            routing_kit_graph.0,
            routing_kit_graph.1,
            routing_kit_graph.2,
            routing_kit_graph.3,
            routing_kit_graph.4,
        );

        (nodes, edges, graph)
    }

    #[test]
    fn test_query_basic_path_without_expansion() {
        let (_nodes, _edges, graph) = create_simple_test_graph();

        // Verify graph was created correctly
        assert!(graph.num_nodes() > 0);
        assert!(graph.num_arcs() > 0);

        // Test that we can query the graph
        let queries_from = vec![0];
        let queries_to = vec![1];
        let queries_departure = vec![0];
        let queries_original_from_edges = vec![0];
        let queries_original_to_edges = vec![1];

        let (paths, distances, _departures) = get_paths_with_dijkstra_queries(
            &queries_from,
            &queries_to,
            &queries_departure,
            &queries_original_from_edges,
            &queries_original_to_edges,
            &graph,
        );

        assert_eq!(paths.len(), 1);
        assert_eq!(distances.len(), 1);
        assert!(distances[0] < FlWeight::INFINITY);
    }

    #[cfg(feature = "expand-sumo-nodes")]
    mod expanded_tests {
        use super::*;

        #[test]
        fn test_query_with_node_expansion() {
            let (_nodes, _edges, graph) = create_simple_test_graph();

            // With node expansion, we should have internal nodes
            // The number of nodes should be 2 * number_of_edges = 2 * 2 = 4
            assert!(graph.num_nodes() >= 4, "Expected at least 4 nodes with expansion, got {}", graph.num_nodes());

            // The number of arcs should include connection edges
            // 2 original edges + at least 1 connection edge = at least 3
            assert!(graph.num_arcs() >= 3, "Expected at least 3 arcs with expansion, got {}", graph.num_arcs());
        }

        #[test]
        fn test_query_path_with_expansion() {
            let (_nodes, _edges, graph) = create_simple_test_graph();

            let queries_from = vec![0];
            let queries_to = vec![2]; // Internal node indices will be different
            let queries_departure = vec![0];
            let queries_original_from_edges = vec![0];
            let queries_original_to_edges = vec![1];

            let (paths, distances, _departures) = get_paths_with_dijkstra_queries(
                &queries_from,
                &queries_to,
                &queries_departure,
                &queries_original_from_edges,
                &queries_original_to_edges,
                &graph,
            );

            assert_eq!(paths.len(), 1);
            assert_eq!(distances.len(), 1);

            // Path should include the from edge, connection edges, and to edge
            if !paths[0].is_empty() {
                assert!(paths[0].len() >= 2, "Expected path with at least 2 edges (from + to)");
            }
        }
    }

    #[cfg(not(feature = "expand-sumo-nodes"))]
    mod non_expanded_tests {
        use super::*;

        #[test]
        fn test_query_without_node_expansion() {
            let (_nodes, _edges, graph) = create_simple_test_graph();

            // Without node expansion, we should have the original nodes count
            // 3 nodes from the test setup
            assert_eq!(graph.num_nodes(), 3, "Expected 3 nodes without expansion, got {}", graph.num_nodes());

            // The number of arcs should be just the original edges (no connections)
            // 2 original edges
            assert_eq!(graph.num_arcs(), 2, "Expected 2 arcs without expansion, got {}", graph.num_arcs());
        }
    }
}
