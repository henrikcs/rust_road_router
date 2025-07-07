use std::{collections::HashMap, path::Path};

use rust_road_router::{
    datastr::graph::floating_time_dependent::RoutingKitTDGraph,
    io::{write_strings_to_file, Store},
};

use crate::{
    sumo::{
        edges::{Edge, EdgesDocumentRoot},
        edges_reader::SumoEdgesReader,
        nodes::{Node, NodesDocumentRoot},
        nodes_reader::SumoNodesReader,
        trips::TripsDocumentRoot,
        trips_reader::SumoTripsReader,
        XmlReader, EDG_XML, NOD_XML, TRIPS_XML,
    },
    DefaultTravelTime, FILE_EDGE_DEFAULT_TRAVEL_TIMES, FILE_EDGE_INDICES_TO_ID, FILE_FIRST_IPP_OF_ARC, FILE_FIRST_OUT, FILE_HEAD, FILE_IPP_DEPARTURE_TIME,
    FILE_IPP_TRAVEL_TIME, FILE_LATITUDE, FILE_LONGITUDE, FILE_QUERIES_DEPARTURE, FILE_QUERIES_FROM, FILE_QUERIES_TO, FILE_QUERY_IDS,
    FILE_QUERY_ORIGINAL_FROM_EDGES, FILE_QUERY_ORIGINAL_TO_EDGES,
};

pub struct FlattenedSumoEdge<'a> {
    from_node_index: u32,
    to_node_index: u32,
    edge_id: &'a String,
    // travel time in seconds
    weight: DefaultTravelTime,
    // length in meters
    length: f64,
}

impl Clone for FlattenedSumoEdge<'_> {
    fn clone(&self) -> Self {
        FlattenedSumoEdge {
            from_node_index: self.from_node_index,
            to_node_index: self.to_node_index,
            edge_id: self.edge_id,
            weight: self.weight,
            length: self.length,
        }
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
pub fn convert_sumo_to_routing_kit_and_queries(input_dir: &Path, input_prefix: &String, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let (nodes, edges, trips) = read_nodes_edges_and_trips_from_plain_xml(input_dir, &input_prefix);

    let (g, edge_ids_to_index, edge_indices_to_id) = get_routing_kit_td_graph_from_sumo(&nodes, &edges);
    let (trip_ids, from, to, departure, original_trip_from_edges, original_trip_to_edges) = get_queries_from_trips(&trips, &edge_ids_to_index);

    let (lat, lon) = nodes.get_latitude_longitude();

    // necessary for creating the TD-CCH. it is also necessary for lat/lon to be f32, otherwise InternalFlowCutterConsole will fail
    lat.write_to(&output_dir.join(FILE_LATITUDE))?;
    lon.write_to(&output_dir.join(FILE_LONGITUDE))?;

    g.0.write_to(&output_dir.join(FILE_FIRST_OUT))?;
    g.1.write_to(&output_dir.join(FILE_HEAD))?;
    g.2.write_to(&output_dir.join(FILE_FIRST_IPP_OF_ARC))?;
    g.3.write_to(&output_dir.join(FILE_IPP_DEPARTURE_TIME))?;
    g.4.write_to(&output_dir.join(FILE_IPP_TRAVEL_TIME))?;

    // extract default weights of all edges and write them to a file
    let edge_default_travel_times: Vec<DefaultTravelTime> = edge_indices_to_id
        .iter()
        .map(|edge| {
            // weight is calculated in method `initialize_edges_for_td_graph`
            edge_ids_to_index.get(edge).unwrap().1.weight
        })
        .collect();

    edge_default_travel_times.write_to(&output_dir.join(FILE_EDGE_DEFAULT_TRAVEL_TIMES))?;

    write_strings_to_file(&output_dir.join(FILE_EDGE_INDICES_TO_ID), &edge_indices_to_id)?;
    write_strings_to_file(&output_dir.join(FILE_QUERY_IDS), &trip_ids)?;
    write_strings_to_file(&output_dir.join(FILE_QUERY_ORIGINAL_FROM_EDGES), &original_trip_from_edges)?;
    write_strings_to_file(&output_dir.join(FILE_QUERY_ORIGINAL_TO_EDGES), &original_trip_to_edges)?;

    from.write_to(&output_dir.join(FILE_QUERIES_FROM))?;
    to.write_to(&output_dir.join(FILE_QUERIES_TO))?;
    departure.write_to(&output_dir.join(FILE_QUERIES_DEPARTURE))?;

    Ok(())
}

pub fn read_nodes_and_edges_from_plain_xml(input_dir: &Path, files_prefix: &String) -> (NodesDocumentRoot, EdgesDocumentRoot) {
    let Ok(edges) = SumoEdgesReader::read(input_dir.join(files_prefix.clone() + EDG_XML).as_path()) else {
        panic!("Edges could not be read.");
    };

    let Ok(nodes) = SumoNodesReader::read(input_dir.join(files_prefix.clone() + NOD_XML).as_path()) else {
        panic!("Edges could not be read.");
    };

    (nodes, edges)
}

pub fn read_nodes_edges_and_trips_from_plain_xml(input_dir: &Path, files_prefix: &String) -> (NodesDocumentRoot, EdgesDocumentRoot, TripsDocumentRoot) {
    let (nodes, edges) = read_nodes_and_edges_from_plain_xml(input_dir, files_prefix);
    let Ok(trips) = SumoTripsReader::read(input_dir.join(files_prefix.clone() + TRIPS_XML).as_path()) else {
        panic!("Trips could not be read.");
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
    edge_id_to_index_map: &HashMap<&String, (usize, FlattenedSumoEdge)>,
) -> (Vec<&'a String>, Vec<u32>, Vec<u32>, Vec<f64>, Vec<&'a String>, Vec<&'a String>) {
    // create a vector of from nodes, to nodes and departure times
    let mut trip_ids = Vec::with_capacity(trips_document_root.vehicles.len());
    let mut from_nodes = Vec::with_capacity(trips_document_root.vehicles.len());
    let mut to_nodes = Vec::with_capacity(trips_document_root.vehicles.len());
    let mut departure_times = Vec::with_capacity(trips_document_root.vehicles.len());
    let mut original_trip_from_edges = Vec::with_capacity(trips_document_root.vehicles.len());
    let mut original_trip_to_edges = Vec::with_capacity(trips_document_root.vehicles.len());

    for veh in &trips_document_root.vehicles {
        trip_ids.push(&veh.id);
        // vehicles go from an edge to an edge, so we need to get the from and to nodes of the edges
        from_nodes.push(edge_id_to_index_map.get(&veh.from).unwrap().1.to_node_index);
        to_nodes.push(edge_id_to_index_map.get(&veh.to).unwrap().1.from_node_index);
        // TODO: maybe add the time it takes from the starting point of the edge to the first node of the query
        departure_times.push(veh.depart);
        original_trip_from_edges.push(&veh.from);
        original_trip_to_edges.push(&veh.to);
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
) -> (RoutingKitTDGraph, HashMap<&'a String, (usize, FlattenedSumoEdge<'a>)>, Vec<&'a String>) {
    // create a floating-td-graph
    // edges should be sorted by node index
    // interpolation points should be initialized with only one timespan (from 0 to end-of-day in seconds)
    // weights are the edge lengths divided by speed (i.e. the travel time).
    // if edge length is not provided, the euclidean distance between the edge's endpoints is used

    // create a map from node id to node index
    // this is used to find the node index of the from and to nodes of each edge

    let node_id_to_index: HashMap<&String, usize> = node_document_root.nodes.iter().enumerate().map(|(index, node)| (&node.id, index)).collect();

    let nodes = &node_document_root.nodes;
    let edges = &edges_document_root.edges;

    let edges_sorted_by_node_index: Vec<FlattenedSumoEdge<'a>> = initialize_edges_for_td_graph(&nodes, &edges, &node_id_to_index);

    let (first_out, head, first_ipp_of_arc, ipp_departure_time, ipp_travel_time) =
        create_implicit_td_graph(nodes.len(), edges.len(), &edges_sorted_by_node_index);

    // with the edge_ids we can write a file containing the edge ids in the order of the edges_sorted_by_node_index
    // this will be used for reconstructing the edges in the TDGraph
    let edge_index_to_edge_id: Vec<&String> = edges_sorted_by_node_index.iter().map(|edge| edge.edge_id).collect();

    let edge_id_to_index: HashMap<&String, (usize, FlattenedSumoEdge<'a>)> = edges_sorted_by_node_index
        .iter()
        .enumerate()
        .map(|(index, e)| (e.edge_id, (index, e.clone())))
        .collect();

    (
        (first_out, head, first_ipp_of_arc, ipp_departure_time, ipp_travel_time),
        edge_id_to_index,
        edge_index_to_edge_id,
    )
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
        ipp_travel_time.push(f64::floor(edge.weight * 1000.0) as u32); // travel time in milliseconds
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
    ipp_departure_time: &[u32],
    ipp_travel_time: &[u32],
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

fn initialize_edges_for_td_graph<'a>(nodes: &'a Vec<Node>, edges: &'a Vec<Edge>, node_id_to_index: &HashMap<&String, usize>) -> Vec<FlattenedSumoEdge<'a>> {
    // edges should be sorted by node index of the tail of the edge
    let mut edges_sorted_by_node_index = Vec::with_capacity(edges.len());
    for edge in edges {
        let &from_node_index = node_id_to_index.get(&edge.from).expect("From node not found in nodes document root");
        let &to_node_index = node_id_to_index.get(&edge.to).expect("To node not found in nodes document root");

        let from_node = &nodes[from_node_index];
        let to_node = &nodes[to_node_index];

        let length = edge.get_length((from_node.x, from_node.y), (to_node.x, to_node.y));

        let weight = length / edge.get_speed();

        let from_node_index = from_node_index as u32;
        let to_node_index = to_node_index as u32;

        dbg!(weight);

        edges_sorted_by_node_index.push(FlattenedSumoEdge {
            from_node_index,
            to_node_index,
            edge_id: &edge.id,
            weight,
            length,
        });
    }

    edges_sorted_by_node_index.sort_by_key(|e| (e.from_node_index, e.to_node_index));

    edges_sorted_by_node_index
}

mod test {
    use std::path::Path;

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
        };

        let edges = EdgesDocumentRoot {
            edges: vec![
                Edge {
                    id: String::from("e1"),
                    from: String::from("n1"),
                    to: String::from("n2"),
                    num_lanes: Some(1),
                    speed: Some(13.9),
                    length: None,
                    lanes: vec![],
                    params: vec![],
                },
                Edge {
                    id: String::from("e2"),
                    from: String::from("n2"),
                    to: String::from("n1"),
                    num_lanes: Some(1),
                    speed: Some(13.9),
                    length: None,
                    lanes: vec![],
                    params: vec![],
                },
            ],
        };

        let (td_graph, edge_ids, edge_indices) = get_routing_kit_td_graph_from_sumo(&nodes, &edges);

        assert_eq!(td_graph.0.len(), 3); // 2 nodes + 1 for the end
        assert_eq!(td_graph.1.len(), 2); // 2 edges
        assert_eq!(td_graph.2.len(), 2 + 1); // 2 edges each having 1 ipp

        assert_eq!(edge_ids.len(), 2); // 2 edges
        assert_eq!(edge_indices[0], "e2");
        assert_eq!(edge_indices[1], "e1");
    }
}
