use std::{collections::HashMap, path::Path};

use rust_road_router::io::{write_strings_to_file, Store};

use crate::{
    sumo::{
        edges::{Edge, EdgesDocumentRoot},
        edges_reader::SumoEdgesReader,
        nodes::{Node, NodesDocumentRoot},
        nodes_reader::SumoNodesReader,
        trips::TripsDocumentRoot,
        trips_reader::SumoTripsReader,
        FileReader, RoutingKitTDGraph, SumoTravelTime, EDG_XML, NOD_XML, VEH_LENGTH,
    },
    SerializedPosition, SerializedTimestamp, SerializedTravelTime, FILE_EDGE_CAPACITIES, FILE_EDGE_DEFAULT_TRAVEL_TIMES, FILE_EDGE_INDICES_TO_ID,
    FILE_FIRST_IPP_OF_ARC, FILE_FIRST_OUT, FILE_HEAD, FILE_IPP_DEPARTURE_TIME, FILE_IPP_TRAVEL_TIME, FILE_LATITUDE, FILE_LONGITUDE, FILE_QUERIES_DEPARTURE,
    FILE_QUERIES_FROM, FILE_QUERIES_TO, FILE_QUERY_IDS, FILE_QUERY_ORIGINAL_FROM_EDGES, FILE_QUERY_ORIGINAL_TO_EDGES,
};

pub struct FlattenedSumoEdge {
    from_node_index: u32,
    to_node_index: u32,
    edge_id: String,
    // travel time in seconds
    weight: SumoTravelTime,
    // length in meters
    length: SumoTravelTime,
    capacity: f64,
}

impl FlattenedSumoEdge {
    pub fn new(from_node_index: u32, to_node_index: u32, edge_id: String, weight: SumoTravelTime, length: SumoTravelTime, capacity: f64) -> Self {
        FlattenedSumoEdge {
            from_node_index,
            to_node_index,
            edge_id,
            weight,
            length,
            capacity,
        }
    }

    pub fn get_edge_id_for_connection(from_connection: &str, to_connection: &str) -> String {
        format!("{}$${}", from_connection, to_connection)
    }
}

/// lowest possible travel time for an edge in seconds
pub const MIN_EDGE_WEIGHT: f64 = 1.0;

impl Clone for FlattenedSumoEdge {
    fn clone(&self) -> Self {
        FlattenedSumoEdge::new(
            self.from_node_index,
            self.to_node_index,
            self.edge_id.clone(),
            self.weight,
            self.length,
            self.capacity,
        )
    }
}

/// Converts SUMO files to a time-dependent graph format defined by RoutingKit
/// creates the following files in the output directory:
/// - first_out: the first outgoing edge for each node
/// - head: the head node of each edge
/// - first_ipp_of_arc: the first interpolation point of each arc
/// - ipp_departure_time: the departure time of each interpolation point
/// - ipp_travel_time: the travel time of each interpolation point
/// - edges_by_id: a file containing the edge ids in the order of the edges in the graph
/// - latitude: the latitude of each node
/// - longitude: the longitude of each node
/// - queries-from: a file containing the from nodes of the queries
/// - queries-to: a file containing the to nodes of the queries
/// - queries-departure: a file containing the departure times of the queries
///
/// With this data, InertialFlowCutterConsole can create a node ranking for the TD-CCH
pub fn convert_sumo_to_routing_kit_and_queries(
    input_dir: &Path,
    input_prefix: &String,
    trips_file: &Path,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let (nodes, edges, trips) = read_nodes_edges_connections_and_trips_from_plain_xml(input_dir, &input_prefix, &trips_file);
    let (g, edge_ids_to_index, edge_indices_to_id) = get_routing_kit_td_graph_from_sumo(&nodes, &edges);
    let (trip_ids, from, to, departure, original_trip_from_edges, original_trip_to_edges) = get_queries_from_trips(&trips, &edge_ids_to_index);

    let (lat, lon) = get_lan_lon_from_nodes(&nodes.nodes);

    // create output_dir, if not exists:
    std::fs::create_dir_all(output_dir)?;

    // necessary for creating the TD-CCH. it is also necessary for lat/lon to be f32, otherwise InternalFlowCutterConsole will fail
    lat.write_to(&output_dir.join(FILE_LATITUDE))?;
    lon.write_to(&output_dir.join(FILE_LONGITUDE))?;

    g.0.write_to(&output_dir.join(FILE_FIRST_OUT))?;
    g.1.write_to(&output_dir.join(FILE_HEAD))?;
    g.2.write_to(&output_dir.join(FILE_FIRST_IPP_OF_ARC))?;
    g.3.write_to(&output_dir.join(FILE_IPP_DEPARTURE_TIME))?;
    g.4.write_to(&output_dir.join(FILE_IPP_TRAVEL_TIME))?;

    // extract default weights of all edges and write them to a file
    let (edge_default_travel_times, capas): (Vec<u32>, Vec<f64>) = edge_indices_to_id
        .iter()
        .map(|edge| {
            // weight is calculated in method `initialize_edges_for_td_graph`
            let e = &edge_ids_to_index.get(edge).unwrap().1;
            let default_tt = (e.weight * 1000.0) as u32; // convert seconds to milliseconds
            let capa = e.capacity;
            (default_tt, capa)
        })
        .collect();

    edge_default_travel_times.write_to(&output_dir.join(FILE_EDGE_DEFAULT_TRAVEL_TIMES))?;
    capas.write_to(&output_dir.join(FILE_EDGE_CAPACITIES))?;

    write_strings_to_file(&output_dir.join(FILE_EDGE_INDICES_TO_ID), &edge_indices_to_id.iter().collect())?;
    write_strings_to_file(&output_dir.join(FILE_QUERY_IDS), &trip_ids)?;

    original_trip_from_edges.write_to(&output_dir.join(FILE_QUERY_ORIGINAL_FROM_EDGES))?;
    original_trip_to_edges.write_to(&output_dir.join(FILE_QUERY_ORIGINAL_TO_EDGES))?;

    from.write_to(&output_dir.join(FILE_QUERIES_FROM))?;
    to.write_to(&output_dir.join(FILE_QUERIES_TO))?;
    departure.write_to(&output_dir.join(FILE_QUERIES_DEPARTURE))?;

    Ok(())
}

pub fn read_nodes_edges_and_connections_from_plain_xml(input_dir: &Path, files_prefix: &String) -> (NodesDocumentRoot, EdgesDocumentRoot) {
    let Ok(edges) = SumoEdgesReader::read(input_dir.join(files_prefix.clone() + EDG_XML).as_path()) else {
        panic!("Edges could not be read form {}.", &input_dir.display());
    };

    let Ok(nodes) = SumoNodesReader::read(input_dir.join(files_prefix.clone() + NOD_XML).as_path()) else {
        panic!("Edges could not be read from {}.", input_dir.display());
    };

    (nodes, edges)
}

pub fn read_nodes_edges_connections_and_trips_from_plain_xml(
    input_dir: &Path,
    files_prefix: &String,
    trips_file: &Path,
) -> (NodesDocumentRoot, EdgesDocumentRoot, TripsDocumentRoot) {
    let (nodes, edges) = read_nodes_edges_and_connections_from_plain_xml(input_dir, files_prefix);
    let Ok(trips) = SumoTripsReader::read(trips_file) else {
        panic!("Trips could not be read from {}.", trips_file.display());
    };

    (nodes, edges, trips)
}

/// Extract queries from the trips document root.
/// The queries from SUMO start and end in edges. However, Catchup is based on nodes.
/// We Transform the edges to nodes by using the to node of the from and and the from node of the to edge.
/// The resulting path of a query then is prepended with the from edge of the query and appended with the to edge of the query to make the path complete.
/// We add to the departure time of the query the time it takes to travel from a random point from the edge to the first node of the query.
pub fn get_queries_from_trips<'a>(
    trips_document_root: &'a TripsDocumentRoot,
    edge_id_to_edge: &HashMap<String, (usize, FlattenedSumoEdge)>,
) -> (Vec<&'a String>, Vec<u32>, Vec<u32>, Vec<SerializedTimestamp>, Vec<u32>, Vec<u32>) {
    // create a vector of from nodes, to nodes and departure times
    let mut trip_ids = Vec::with_capacity(trips_document_root.trips.len());
    let mut from_nodes = Vec::with_capacity(trips_document_root.trips.len());
    let mut to_nodes = Vec::with_capacity(trips_document_root.trips.len());
    let mut departure_times = Vec::with_capacity(trips_document_root.trips.len());
    let mut original_trip_from_edges = Vec::with_capacity(trips_document_root.trips.len());
    let mut original_trip_to_edges = Vec::with_capacity(trips_document_root.trips.len());

    for veh in &trips_document_root.trips {
        trip_ids.push(&veh.id);
        // vehicles go from an edge to an edge, so we need to get the from and to nodes of the edges
        let (from_index, from_edge) = edge_id_to_edge
            .get(&veh.from)
            .unwrap_or_else(|| panic!("From edge {} not found in edge_id_to_index_map", veh.from));
        from_nodes.push(from_edge.to_node_index);

        let (to_index, to_edge) = edge_id_to_edge
            .get(&veh.to)
            .unwrap_or_else(|| panic!("To edge {} not found in edge_id_to_index_map", veh.to));
        to_nodes.push(to_edge.from_node_index);

        departure_times.push((veh.depart * 1000.0) as SerializedTimestamp); // convert seconds to milliseconds
        original_trip_from_edges.push(*from_index as u32);
        original_trip_to_edges.push(*to_index as u32);
    }

    (
        trip_ids,
        from_nodes,
        to_nodes,
        departure_times,
        original_trip_from_edges,
        original_trip_to_edges,
    )
}

pub fn get_routing_kit_td_graph_from_sumo<'a>(
    node_document_root: &'a NodesDocumentRoot,
    edges_document_root: &'a EdgesDocumentRoot,
) -> (RoutingKitTDGraph, HashMap<String, (usize, FlattenedSumoEdge)>, Vec<String>) {
    // create a floating-td-graph
    // edges should be sorted by node index
    // interpolation points should be initialized with only one timespan (from 0 to end-of-day in seconds)
    // weights are the edge lengths divided by speed (i.e. the travel time).
    // if edge length is not provided, the euclidean distance between the edge's endpoints is used

    let nodes = &node_document_root.nodes;
    let edges = &edges_document_root.edges;
    let edges_sorted_by_node_index = initialize_edges_for_td_graph(&nodes, &edges);

    let (first_out, head, first_ipp_of_arc, ipp_departure_time, ipp_travel_time) =
        create_implicit_td_graph(nodes.len(), edges_sorted_by_node_index.len(), &edges_sorted_by_node_index);

    // with the edge_ids we can write a file containing the edge ids in the order of the edges_sorted_by_node_index
    // this will be used for reconstructing the edges in the TDGraph
    let edge_index_to_edge_id: Vec<String> = edges_sorted_by_node_index.iter().map(|edge| edge.edge_id.clone()).collect();

    let edge_id_to_index: HashMap<String, (usize, FlattenedSumoEdge)> = edges_sorted_by_node_index
        .iter()
        .enumerate()
        .map(|(index, e)| (e.edge_id.clone(), (index, e.clone())))
        .collect();

    (
        (first_out, head, first_ipp_of_arc, ipp_departure_time, ipp_travel_time),
        edge_id_to_index,
        edge_index_to_edge_id,
    )
}

fn get_lan_lon_from_nodes(nodes: &Vec<Node>) -> (Vec<SerializedPosition>, Vec<SerializedPosition>) {
    let lat: Vec<SerializedPosition> = nodes.iter().map(|n| n.y as SerializedPosition).collect();
    let lon: Vec<SerializedPosition> = nodes.iter().map(|n| n.x as SerializedPosition).collect();

    (lat, lon)
}

fn create_implicit_td_graph(number_of_nodes: usize, number_of_edges: usize, edges_sorted_by_node_index: &Vec<FlattenedSumoEdge>) -> RoutingKitTDGraph {
    // ipp_departure time is 0 for each edge, and ipp_travel_time is the weight of the edge
    let mut first_out = Vec::with_capacity(number_of_nodes + 1);
    let mut head = Vec::with_capacity(number_of_edges);
    let mut first_ipp_of_arc = Vec::with_capacity(number_of_edges + 1);
    let mut ipp_departure_time = Vec::with_capacity(number_of_edges);
    let mut ipp_travel_time = Vec::with_capacity(number_of_edges);

    for edge in edges_sorted_by_node_index {
        // skip nodes which do not have outgoing edges
        while first_out.len() <= edge.from_node_index as usize {
            first_out.push(head.len() as u32);
        }
        // skip edges which do not have interpolation points
        while first_ipp_of_arc.len() <= head.len() {
            first_ipp_of_arc.push(ipp_departure_time.len() as u32);
        }

        // add the head of the edge
        head.push(edge.to_node_index);

        // add the ipp departure time and travel time
        ipp_departure_time.push(0); // departure time is 0 for all edges
        ipp_travel_time.push((edge.weight * 1000.0) as SerializedTravelTime); // convert seconds to milliseconds
                                                                              // travel time in milliseconds
    }

    // a loop is necessary in the case that the last node has no outgoing edges
    while first_out.len() <= number_of_nodes {
        first_out.push(head.len() as u32); // add the end of the first_out array
    }

    // it should not be necessary do a loop because the last edge is guaranteed to have been added
    first_ipp_of_arc.push(number_of_edges as u32);

    assert_correct_number_of_vec_items(
        number_of_nodes,
        number_of_edges,
        &first_out,
        &head,
        &first_ipp_of_arc,
        &ipp_departure_time,
        &ipp_travel_time,
    );

    (first_out, head, first_ipp_of_arc, ipp_departure_time, ipp_travel_time)
}

fn assert_correct_number_of_vec_items(
    number_of_nodes: usize,
    number_of_edges: usize,
    first_out: &[u32],
    head: &[u32],
    first_ipp_of_arc: &[u32],
    ipp_departure_time: &[SerializedTimestamp],
    ipp_travel_time: &[SerializedTravelTime],
) {
    assert!(
        first_out.len() == number_of_nodes + 1,
        "The length of first_out does not match the number of nodes + 1. This is a bug.",
    );

    assert!(
        head.len() == number_of_edges,
        "The length of head does not match the number of edges. This is a bug.",
    );

    assert!(
        first_ipp_of_arc.len() == number_of_edges + 1,
        "The length of first_ipp_of_arc does not match the number of edges + 1. This is a bug.",
    );

    assert!(
        ipp_departure_time.len() == number_of_edges,
        "The length of ipp_departure_time does not match the number of edges. This is a bug.",
    );

    assert!(
        ipp_travel_time.len() == number_of_edges,
        "The length of ipp_travel_time does not match the number of edges. This is a bug.",
    );
}

fn initialize_edges_for_td_graph(nodes: &Vec<Node>, edges: &Vec<Edge>) -> Vec<FlattenedSumoEdge> {
    let node_id_to_index: HashMap<&String, usize> = nodes.iter().enumerate().map(|(index, node)| (&node.id, index)).collect();

    let mut edges_sorted_by_node_index = Vec::with_capacity(edges.len());
    for edge in edges {
        let &from_node_index = node_id_to_index.get(&edge.from).expect("From node not found in nodes document root");
        let &to_node_index = node_id_to_index.get(&edge.to).expect("To node not found in nodes document root");

        let from_node = &nodes[from_node_index];
        let to_node = &nodes[to_node_index];

        let length = edge.get_length((from_node.x, from_node.y), (to_node.x, to_node.y));

        let weight = f64::max(f64::max(length - 1.0 * VEH_LENGTH, 0.0) / edge.get_speed(), MIN_EDGE_WEIGHT);

        let from_node_index = from_node_index as u32;
        let to_node_index = to_node_index as u32;

        edges_sorted_by_node_index.push(FlattenedSumoEdge::new(
            from_node_index,
            to_node_index,
            edge.id.clone(),
            weight,
            length,
            edge.get_capacity(),
        ));
    }

    edges_sorted_by_node_index.sort_by_key(|e| (e.from_node_index, e.to_node_index));

    edges_sorted_by_node_index
}

/// For each edge, we create two nodes with the id "<node_id>\n<edge_id>" with weight MIN_EDGE_WEIGHT.
/// The position should be the same as the original node.
/// This is necessary to model turn restrictions and turn costs.
fn _expand_nodes(nodes: &Vec<Node>, edges: &Vec<Edge>) -> Vec<Node> {
    // create a temporaray map from node id to node
    let node_id_to_node: HashMap<&String, &Node> = nodes.iter().map(|node| (&node.id, node)).collect();

    let mut expanded_nodes = Vec::with_capacity(edges.len() * 2);
    for edge in edges {
        expanded_nodes.push(Node {
            id: Node::get_node_id_for_internal_node(&edge.from, &edge.id),
            x: node_id_to_node.get(&edge.from).unwrap().x,
            y: node_id_to_node.get(&edge.from).unwrap().y,
        });
        expanded_nodes.push(Node {
            id: Node::get_node_id_for_internal_node(&edge.to, &edge.id),
            x: node_id_to_node.get(&edge.to).unwrap().x,
            y: node_id_to_node.get(&edge.to).unwrap().y,
        });
    }
    expanded_nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::sumo::{
        edges::{Edge, EdgesDocumentRoot},
        nodes::NodesDocumentRoot,
    };

    #[test]
    fn test_convert_sumo_to_td_graph() {
        let nodes = NodesDocumentRoot {
            nodes: vec![
                crate::sumo::nodes::Node {
                    id: String::from("n2"),
                    x: 0.0,
                    y: 0.0,
                },
                crate::sumo::nodes::Node {
                    id: String::from("n1"),
                    x: 1.0,
                    y: 1.0,
                },
            ],
            location: None,
        };

        let edges = EdgesDocumentRoot {
            edges: vec![
                Edge {
                    id: String::from("e2"),
                    from: String::from("n2"),
                    to: String::from("n1"),
                    num_lanes: Some(1),
                    speed: Some(13.9),
                    length: None,
                    lanes: vec![],
                    params: vec![],
                    priority: Some(-1),
                },
                Edge {
                    id: String::from("e1"),
                    from: String::from("n1"),
                    to: String::from("n2"),
                    num_lanes: Some(1),
                    speed: Some(13.9),
                    length: None,
                    lanes: vec![],
                    params: vec![],
                    priority: Some(-1),
                },
            ],
        };

        let (td_graph, _, _) = get_routing_kit_td_graph_from_sumo(&nodes, &edges);

        assert_eq!(td_graph.0.len(), 5); // 2 edges = 4 nodes + 1 for the end
        assert_eq!(td_graph.1.len(), 2); // 2 edges
        assert_eq!(td_graph.2.len(), 2 + 1); // 2 edges each having 1 ipp

        // Check expanded node names using Node::get_node_id_for_internal_node
        use crate::sumo::nodes::Node;
        let expected_node_names = vec![
            Node::get_node_id_for_internal_node("n2", "e2"),
            Node::get_node_id_for_internal_node("n1", "e2"),
            Node::get_node_id_for_internal_node("n1", "e1"),
            Node::get_node_id_for_internal_node("n2", "e1"),
        ];
        let actual_node_names: Vec<_> = nodes.nodes.iter().map(|n| n.id.as_str()).collect();
        let expected_node_names: Vec<_> = expected_node_names.iter().map(|s| s.as_str()).collect();
        assert_eq!(actual_node_names, expected_node_names);
    }

    #[test]
    fn test_td_graph_with_some_connections() {
        // 2 nodes, 2 edges, 1 connection from e1 to e2
        let nodes = NodesDocumentRoot {
            nodes: vec![
                crate::sumo::nodes::Node {
                    id: String::from("n2"),
                    x: 0.0,
                    y: 0.0,
                },
                crate::sumo::nodes::Node {
                    id: String::from("n1"),
                    x: 1.0,
                    y: 1.0,
                },
            ],
            location: None,
        };

        let edges = EdgesDocumentRoot {
            edges: vec![
                Edge {
                    id: String::from("e2"),
                    from: String::from("n2"),
                    to: String::from("n1"),
                    num_lanes: Some(1),
                    speed: Some(13.9),
                    length: None,
                    lanes: vec![],
                    params: vec![],
                    priority: Some(-1),
                },
                Edge {
                    id: String::from("e1"),
                    from: String::from("n1"),
                    to: String::from("n2"),
                    num_lanes: Some(1),
                    speed: Some(13.9),
                    length: None,
                    lanes: vec![],
                    params: vec![],
                    priority: Some(-1),
                },
            ],
        };

        let (td_graph, _, edge_ids) = get_routing_kit_td_graph_from_sumo(&nodes, &edges);

        // 2 original edges + 1 connection edge
        assert_eq!(td_graph.1.len(), 3);
        // The edge_ids should contain the connection edge id
        assert!(edge_ids
            .iter()
            .any(|id| id.contains(&FlattenedSumoEdge::get_edge_id_for_connection("e1", "e2"))));

        // Check expanded node names using Node::get_node_id_for_internal_node
        use crate::sumo::nodes::Node;
        let expected_node_names = vec![
            Node::get_node_id_for_internal_node("n2", "e2"),
            Node::get_node_id_for_internal_node("n1", "e2"),
            Node::get_node_id_for_internal_node("n1", "e1"),
            Node::get_node_id_for_internal_node("n2", "e1"),
        ];
        let actual_node_names: Vec<_> = nodes.nodes.iter().map(|n| n.id.as_str()).collect();
        let expected_node_names: Vec<_> = expected_node_names.iter().map(|s| s.as_str()).collect();
        assert_eq!(actual_node_names, expected_node_names);
    }

    #[test]
    fn test_td_graph_with_multi_connections() {
        // 2 nodes, 2 edges, connections from every edge to every edge (including self)
        let nodes = NodesDocumentRoot {
            nodes: vec![
                crate::sumo::nodes::Node {
                    id: String::from("n2"),
                    x: 0.0,
                    y: 0.0,
                },
                crate::sumo::nodes::Node {
                    id: String::from("n1"),
                    x: 1.0,
                    y: 1.0,
                },
            ],
            location: None,
        };

        let edges = EdgesDocumentRoot {
            edges: vec![
                Edge {
                    id: String::from("e2"),
                    from: String::from("n2"),
                    to: String::from("n1"),
                    num_lanes: Some(1),
                    speed: Some(13.9),
                    length: None,
                    lanes: vec![],
                    params: vec![],
                    priority: Some(-1),
                },
                Edge {
                    id: String::from("e1"),
                    from: String::from("n1"),
                    to: String::from("n2"),
                    num_lanes: Some(1),
                    speed: Some(13.9),
                    length: None,
                    lanes: vec![],
                    params: vec![],
                    priority: Some(-1),
                },
            ],
        };

        let (td_graph, _, edge_ids) = get_routing_kit_td_graph_from_sumo(&nodes, &edges);

        // 2 original edges + 2 connection edges (2 from each edge to both edges)
        assert_eq!(td_graph.1.len(), 4);

        // All possible connection edge ids should be present
        let expected_connections = vec![
            FlattenedSumoEdge::get_edge_id_for_connection("e1", "e2"),
            FlattenedSumoEdge::get_edge_id_for_connection("e2", "e1"),
        ];
        for conn in expected_connections {
            assert!(edge_ids.iter().any(|id| id.contains(&conn)), "Missing connection edge id: {}", conn);
        }

        // Check expanded node names using Node::get_node_id_for_internal_node
        use crate::sumo::nodes::Node;
        let expected_node_names = vec![
            Node::get_node_id_for_internal_node("n2", "e2"),
            Node::get_node_id_for_internal_node("n1", "e2"),
            Node::get_node_id_for_internal_node("n1", "e1"),
            Node::get_node_id_for_internal_node("n2", "e1"),
        ];
        let actual_node_names: Vec<_> = nodes.nodes.iter().map(|n| n.id.as_str()).collect();
        let expected_node_names: Vec<_> = expected_node_names.iter().map(|s| s.as_str()).collect();
        assert_eq!(actual_node_names, expected_node_names);
    }
}
