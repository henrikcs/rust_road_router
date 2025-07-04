use std::{collections::HashMap, path::Path};

use rust_road_router::datastr::graph::floating_time_dependent::{ImplicitTDGraph, TDGraph};

use crate::sumo::{
    edges::{Edge, EdgesDocumentRoot},
    edges_reader::SumoEdgesReader,
    nodes::{Node, NodesDocumentRoot},
    nodes_reader::SumoNodesReader,
    trips::TripsDocumentRoot,
    trips_reader::SumoTripsReader,
    XmlReader,
};

type FlattenedSumoEdge<'a> = (u32, u32, &'a String, f64, f64); // (from_node_index, to_node_index, edge_id, weight, length)

const EDG_XML: &str = ".edg.xml";
const NOD_XML: &str = ".nod.xml";
const CON_XML: &str = ".con.xml";
const TRIPS_XML: &str = ".trips.xml";

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
    let Ok(trips) = SumoTripsReader::read(input_dir.join(files_prefix.clone() + NOD_XML).as_path()) else {
        panic!("Trips could not be read.");
    };

    (nodes, edges, trips)
}

pub fn convert_sumo_to_td_graph<'a>(
    node_document_root: &'a NodesDocumentRoot,
    edges_document_root: &'a EdgesDocumentRoot,
) -> (ImplicitTDGraph, Vec<&'a String>) {
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

    let edges_sorted_by_node_index = sort_edges_by_node_index(&nodes, &edges, &node_id_to_index);

    let (first_out, head, first_ipp_of_arc, ipp_departure_time, ipp_travel_time) =
        create_implicit_td_graph(nodes.len(), edges.len(), &edges_sorted_by_node_index);

    // with the edge_ids we can write a file containing the edge ids in the order of the edges_sorted_by_node_index
    // this will be used for reconstructing the edges in the TDGraph
    let edge_ids: Vec<&String> = edges_sorted_by_node_index.iter().map(|(_, _, id, _, _)| *id).collect();

    ((first_out, head, first_ipp_of_arc, ipp_departure_time, ipp_travel_time), edge_ids)
}

fn create_implicit_td_graph(number_of_nodes: usize, number_of_edges: usize, edges_sorted_by_node_index: &Vec<FlattenedSumoEdge>) -> ImplicitTDGraph {
    // ipp_departure time is 0 for each edge, and ipp_travel_time is the weight of the edge
    let mut first_out = Vec::with_capacity(number_of_nodes + 1);
    let mut head = Vec::with_capacity(number_of_edges);
    let mut first_ipp_of_arc = Vec::with_capacity(number_of_edges + 1);
    let mut ipp_departure_time = Vec::with_capacity(number_of_edges);
    let mut ipp_travel_time = Vec::with_capacity(number_of_edges);

    for &(from_node_index, to_node_index, _edge_id, weight, _length) in edges_sorted_by_node_index {
        // ensure that first_out has enough space for the from_node_index
        while first_out.len() <= from_node_index as usize {
            first_out.push(head.len() as u32);
        }
        // ensure that first_ipp_of_arc has enough space for the current edge
        while first_ipp_of_arc.len() <= head.len() {
            first_ipp_of_arc.push(ipp_departure_time.len() as u32);
        }

        // add the head of the edge
        head.push(to_node_index);

        // add the ipp departure time and travel time
        ipp_departure_time.push(0); // departure time is 0 for all edges
        ipp_travel_time.push(f64::floor(weight * 1000.0) as u32); // travel time in milliseconds
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

fn sort_edges_by_node_index<'a>(nodes: &'a Vec<Node>, edges: &'a Vec<Edge>, node_id_to_index: &HashMap<&String, usize>) -> Vec<FlattenedSumoEdge<'a>> {
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

        edges_sorted_by_node_index.push((from_node_index, to_node_index, &edge.id, weight, length));
    }

    edges_sorted_by_node_index.sort_by_key(|(from, to, _, _, _)| (*from, *to));

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

        let (td_graph, edge_ids) = convert_sumo_to_td_graph(&nodes, &edges);

        assert_eq!(td_graph.0.len(), 3); // 2 nodes + 1 for the end
        assert_eq!(td_graph.1.len(), 2); // 2 edges
        assert_eq!(td_graph.2.len(), 2 + 1); // 2 edges each having 1 ipp

        assert_eq!(edge_ids.len(), 2); // 2 edges
        assert_eq!(edge_ids[0], "e2");
        assert_eq!(edge_ids[1], "e1");
    }
}
