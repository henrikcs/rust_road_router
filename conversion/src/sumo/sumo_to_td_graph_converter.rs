use std::{collections::HashMap, path::Path};

#[cfg(feature = "expand-sumo-nodes")]
use std::collections::HashSet;

use rust_road_router::io::{write_strings_to_file, Store};

use crate::{
    sumo::{
        edges::{Edge, EdgesDocumentRoot},
        edges_reader::SumoEdgesReader,
        nodes::{Node, NodesDocumentRoot},
        nodes_reader::SumoNodesReader,
        trips::TripsDocumentRoot,
        trips_reader::SumoTripsReader,
        FileReader, RoutingKitTDGraph, SumoTimestamp, SumoTravelTime, EDG_XML, NOD_XML,
    },
    SerializedPosition, SerializedTimestamp, SerializedTravelTime, FILE_EDGE_CAPACITIES, FILE_EDGE_DEFAULT_TRAVEL_TIMES, FILE_EDGE_INDICES_TO_ID,
    FILE_EDGE_LANES, FILE_EDGE_LENGTHS, FILE_EDGE_SPEEDS, FILE_FIRST_IPP_OF_ARC, FILE_FIRST_OUT, FILE_HEAD, FILE_IPP_DEPARTURE_TIME, FILE_IPP_TRAVEL_TIME,
    FILE_LATITUDE, FILE_LONGITUDE, FILE_QUERIES_DEPARTURE, FILE_QUERIES_FROM, FILE_QUERIES_TO, FILE_QUERY_IDS, FILE_QUERY_ORIGINAL_FROM_EDGES,
    FILE_QUERY_ORIGINAL_TO_EDGES, GLOBAL_FREE_FLOW_SPEED_FACTOR, MIN_EDGE_WEIGHT, SUMO_DEFAULT_SPEED,
};

#[cfg(feature = "expand-sumo-nodes")]
use crate::sumo::{
    connections::{Connection, ConnectionsDocumentRoot},
    connections_reader::SumoConnectionsReader,
    CON_XML,
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
    lanes: u32,
    speed: f64,
}

impl FlattenedSumoEdge {
    pub fn new(
        from_node_index: u32,
        to_node_index: u32,
        edge_id: String,
        weight: SumoTravelTime,
        length: SumoTravelTime,
        capacity: f64,
        lanes: u32,
        speed: f64,
    ) -> Self {
        FlattenedSumoEdge {
            from_node_index,
            to_node_index,
            edge_id,
            weight,
            length,
            capacity,
            lanes,
            speed,
        }
    }

    pub fn get_edge_id_for_connection(from_connection: &str, to_connection: &str) -> String {
        format!("{}$${}", from_connection, to_connection)
    }
}

impl Clone for FlattenedSumoEdge {
    fn clone(&self) -> Self {
        FlattenedSumoEdge::new(
            self.from_node_index,
            self.to_node_index,
            self.edge_id.clone(),
            self.weight,
            self.length,
            self.capacity,
            self.lanes,
            self.speed,
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
    begin: Option<SumoTimestamp>,
    end: Option<SumoTimestamp>,
    interval: Option<SumoTimestamp>,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "expand-sumo-nodes")]
    let (nodes, edges, connections, trips) = read_nodes_edges_connections_and_trips_from_plain_xml(input_dir, &input_prefix, &trips_file);
    #[cfg(feature = "expand-sumo-nodes")]
    let (g, expanded_nodes, edge_ids_to_index, edge_indices_to_id) = get_routing_kit_td_graph_from_sumo(&nodes, &edges, &connections, begin, end, interval);
    #[cfg(feature = "expand-sumo-nodes")]
    let all_nodes = expanded_nodes;

    #[cfg(not(feature = "expand-sumo-nodes"))]
    let (nodes, edges, trips) = read_nodes_edges_connections_and_trips_from_plain_xml(input_dir, &input_prefix, &trips_file);
    #[cfg(not(feature = "expand-sumo-nodes"))]
    let (g, edge_ids_to_index, edge_indices_to_id) = get_routing_kit_td_graph_from_sumo(&nodes, &edges, begin, end, interval);
    #[cfg(not(feature = "expand-sumo-nodes"))]
    let all_nodes = nodes.nodes;

    let (trip_ids, from, to, departure, original_trip_from_edges, original_trip_to_edges) = get_queries_from_trips(&trips, &edge_ids_to_index);
    let (lat, lon) = get_lan_lon_from_nodes(&all_nodes);

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
    let (edge_default_travel_times, capas, lengths, lanes, speeds): (Vec<u32>, Vec<f64>, Vec<f64>, Vec<u32>, Vec<f64>) = edge_indices_to_id
        .iter()
        .map(|edge| {
            // weight is calculated in method `initialize_edges_for_td_graph`
            let e = &edge_ids_to_index.get(edge).unwrap().1;
            let default_tt = (e.weight * 1000.0) as u32; // convert seconds to milliseconds
            let capa = e.capacity;
            let length = e.length;
            let lanes = e.lanes;
            let speeds = e.speed;

            (default_tt, capa, length, lanes, speeds)
        })
        .collect();

    edge_default_travel_times.write_to(&output_dir.join(FILE_EDGE_DEFAULT_TRAVEL_TIMES))?;
    lengths.write_to(&output_dir.join(FILE_EDGE_LENGTHS))?;
    lanes.write_to(&output_dir.join(FILE_EDGE_LANES))?;
    speeds.write_to(&output_dir.join(FILE_EDGE_SPEEDS))?;
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

#[cfg(feature = "expand-sumo-nodes")]
pub fn read_nodes_edges_and_connections_from_plain_xml(
    input_dir: &Path,
    files_prefix: &String,
) -> (NodesDocumentRoot, EdgesDocumentRoot, ConnectionsDocumentRoot) {
    let Ok(edges) = SumoEdgesReader::read(input_dir.join(files_prefix.clone() + EDG_XML).as_path()) else {
        panic!("Edges could not be read form {}.", &input_dir.display());
    };

    let Ok(nodes) = SumoNodesReader::read(input_dir.join(files_prefix.clone() + NOD_XML).as_path()) else {
        panic!("Edges could not be read from {}.", input_dir.display());
    };

    let Ok(connections) = SumoConnectionsReader::read(input_dir.join(files_prefix.clone() + CON_XML).as_path()) else {
        panic!("Connections could not be read from {}.", input_dir.display());
    };

    (nodes, edges, connections)
}

#[cfg(not(feature = "expand-sumo-nodes"))]
pub fn read_nodes_edges_and_connections_from_plain_xml(input_dir: &Path, files_prefix: &String) -> (NodesDocumentRoot, EdgesDocumentRoot) {
    let Ok(edges) = SumoEdgesReader::read(input_dir.join(files_prefix.clone() + EDG_XML).as_path()) else {
        panic!("Edges could not be read form {}.", &input_dir.display());
    };

    let Ok(nodes) = SumoNodesReader::read(input_dir.join(files_prefix.clone() + NOD_XML).as_path()) else {
        panic!("Edges could not be read from {}.", input_dir.display());
    };

    (nodes, edges)
}

#[cfg(feature = "expand-sumo-nodes")]
pub fn read_nodes_edges_connections_and_trips_from_plain_xml(
    input_dir: &Path,
    files_prefix: &String,
    trips_file: &Path,
) -> (NodesDocumentRoot, EdgesDocumentRoot, ConnectionsDocumentRoot, TripsDocumentRoot) {
    let (nodes, edges, connections) = read_nodes_edges_and_connections_from_plain_xml(input_dir, files_prefix);
    let Ok(trips) = SumoTripsReader::read(trips_file) else {
        panic!("Trips could not be read from {}.", trips_file.display());
    };

    (nodes, edges, connections, trips)
}

#[cfg(not(feature = "expand-sumo-nodes"))]
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
#[cfg(feature = "expand-sumo-nodes")]
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
        // vehicles go from an edge to an edge
        // with node expansion, we use internal nodes: the "to" internal node of from_edge and the "from" internal node of to_edge
        let (from_index, from_edge) = edge_id_to_edge
            .get(&veh.from)
            .unwrap_or_else(|| panic!("From edge {} not found in edge_id_to_index_map", veh.from));

        // In expanded mode, we want the internal node at the tail of the from_edge
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

#[cfg(not(feature = "expand-sumo-nodes"))]
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

#[cfg(feature = "expand-sumo-nodes")]
pub fn get_routing_kit_td_graph_from_sumo<'a>(
    node_document_root: &'a NodesDocumentRoot,
    edges_document_root: &'a EdgesDocumentRoot,
    connections_document_root: &'a ConnectionsDocumentRoot,
    begin: Option<SumoTimestamp>,
    end: Option<SumoTimestamp>,
    interval: Option<SumoTimestamp>,
) -> (RoutingKitTDGraph, Vec<Node>, HashMap<String, (usize, FlattenedSumoEdge)>, Vec<String>) {
    // create a floating-td-graph
    // edges should be sorted by node index
    // interpolation points should be initialized with only one timespan (from 0 to end-of-day in seconds)
    // weights are the edge lengths divided by speed (i.e. the travel time).
    // if edge length is not provided, the euclidean distance between the edge's endpoints is used

    let nodes = &node_document_root.nodes;
    let edges = &edges_document_root.edges;
    let connections = &connections_document_root.connections;
    let (expanded_nodes, edges_sorted_by_node_index) = initialize_edges_for_td_graph_with_expansion(&nodes, &edges, &connections);

    let (first_out, head, first_ipp_of_arc, ipp_departure_time, ipp_travel_time) = create_implicit_td_graph(
        expanded_nodes.len(),
        edges_sorted_by_node_index.len(),
        &edges_sorted_by_node_index,
        get_departure_times(begin, end, interval).as_ref(),
    );

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
        expanded_nodes,
        edge_id_to_index,
        edge_index_to_edge_id,
    )
}

#[cfg(not(feature = "expand-sumo-nodes"))]
pub fn get_routing_kit_td_graph_from_sumo<'a>(
    node_document_root: &'a NodesDocumentRoot,
    edges_document_root: &'a EdgesDocumentRoot,
    begin: Option<SumoTimestamp>,
    end: Option<SumoTimestamp>,
    interval: Option<SumoTimestamp>,
) -> (RoutingKitTDGraph, HashMap<String, (usize, FlattenedSumoEdge)>, Vec<String>) {
    // create a floating-td-graph
    // edges should be sorted by node index
    // interpolation points should be initialized with only one timespan (from 0 to end-of-day in seconds)
    // weights are the edge lengths divided by speed (i.e. the travel time).
    // if edge length is not provided, the euclidean distance between the edge's endpoints is used

    let nodes = &node_document_root.nodes;
    let edges = &edges_document_root.edges;
    let edges_sorted_by_node_index = initialize_edges_for_td_graph(&nodes, &edges);

    let (first_out, head, first_ipp_of_arc, ipp_departure_time, ipp_travel_time) = create_implicit_td_graph(
        nodes.len(),
        edges_sorted_by_node_index.len(),
        &edges_sorted_by_node_index,
        get_departure_times(begin, end, interval).as_ref(),
    );

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

fn get_departure_times(begin: Option<SumoTimestamp>, end: Option<SumoTimestamp>, interval: Option<SumoTimestamp>) -> Vec<u32> {
    let begin = begin.unwrap_or(0.0);
    let end = end.unwrap_or(86400.0); // default end is end of day in seconds
    let interval = interval.unwrap_or(end - begin); // default interval is the whole time span

    let mut departure_times = Vec::new();
    let mut current_time = begin;
    while current_time < end {
        departure_times.push(current_time as u32);
        current_time += interval;
    }

    departure_times
}

fn get_lan_lon_from_nodes(nodes: &Vec<Node>) -> (Vec<SerializedPosition>, Vec<SerializedPosition>) {
    let lat: Vec<SerializedPosition> = nodes.iter().map(|n| n.y as SerializedPosition).collect();
    let lon: Vec<SerializedPosition> = nodes.iter().map(|n| n.x as SerializedPosition).collect();

    (lat, lon)
}

fn create_implicit_td_graph(
    number_of_nodes: usize,
    number_of_edges: usize,
    edges_sorted_by_node_index: &Vec<FlattenedSumoEdge>,
    departure_times: &Vec<u32>,
) -> RoutingKitTDGraph {
    // ipp_departure time is 0 for each edge, and ipp_travel_time is the weight of the edge
    let mut first_out = Vec::with_capacity(number_of_nodes + 1);
    let mut head = Vec::with_capacity(number_of_edges);
    let mut first_ipp_of_arc = Vec::with_capacity(number_of_edges + 1);
    let mut ipp_departure_time = Vec::with_capacity(number_of_edges * departure_times.len());
    let mut ipp_travel_time = Vec::with_capacity(number_of_edges * departure_times.len());

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
        for departure_time in departure_times {
            ipp_departure_time.push(*departure_time);
            // set initial weight in all departure times to the default weight of the edge
            // convert seconds to milliseconds
            ipp_travel_time.push((edge.weight * 1000.0) as SerializedTravelTime);
        }
    }

    // a loop is necessary in the case that the last node has no outgoing edges
    while first_out.len() <= number_of_nodes {
        first_out.push(head.len() as u32); // add the end of the first_out array
    }

    // it should not be necessary do a loop because the last edge is guaranteed to have been added
    first_ipp_of_arc.push(ipp_departure_time.len() as u32);

    assert_correct_number_of_vec_items(
        number_of_nodes,
        number_of_edges,
        &first_out,
        &head,
        &first_ipp_of_arc,
        &ipp_departure_time,
        &ipp_travel_time,
        departure_times,
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
    departure_times: &[u32],
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
        ipp_departure_time.len() == number_of_edges * departure_times.len(),
        "The length of ipp_departure_time does not match the number of edges. This is a bug.",
    );

    assert!(
        ipp_travel_time.len() == number_of_edges * departure_times.len(),
        "The length of ipp_travel_time does not match the number of edges. This is a bug.",
    );
}
#[cfg(not(feature = "expand-sumo-nodes"))]
fn initialize_edges_for_td_graph(nodes: &Vec<Node>, edges: &Vec<Edge>) -> Vec<FlattenedSumoEdge> {
    let node_id_to_index: HashMap<&String, usize> = nodes.iter().enumerate().map(|(index, node)| (&node.id, index)).collect();

    let mut edges_sorted_by_node_index = Vec::with_capacity(edges.len());
    for edge in edges {
        let &from_node_index = node_id_to_index.get(&edge.from).expect("From node not found in nodes document root");
        let &to_node_index = node_id_to_index.get(&edge.to).expect("To node not found in nodes document root");

        let from_node = &nodes[from_node_index];
        let to_node = &nodes[to_node_index];

        let length = edge.get_length((from_node.x, from_node.y), (to_node.x, to_node.y));

        // weight is the default weight, which is the length divided by the free-flow speed
        let weight = f64::max(length / (edge.get_speed() * GLOBAL_FREE_FLOW_SPEED_FACTOR), MIN_EDGE_WEIGHT);

        let from_node_index = from_node_index as u32;
        let to_node_index = to_node_index as u32;

        edges_sorted_by_node_index.push(FlattenedSumoEdge::new(
            from_node_index,
            to_node_index,
            edge.id.clone(),
            weight,
            length,
            edge.get_capacity(),
            edge.num_lanes.unwrap_or(1),
            edge.speed.unwrap_or(SUMO_DEFAULT_SPEED),
        ));
    }

    edges_sorted_by_node_index.sort_by_key(|e| (e.from_node_index, e.to_node_index));

    edges_sorted_by_node_index
}

#[cfg(feature = "expand-sumo-nodes")]
fn initialize_edges_for_td_graph_with_expansion(nodes: &Vec<Node>, edges: &Vec<Edge>, connections: &Vec<Connection>) -> (Vec<Node>, Vec<FlattenedSumoEdge>) {
    // create a map from node id to node index
    // this is used to find the node index of the from and to nodes of each edge
    let expanded_nodes = expand_nodes(nodes, edges);

    let node_id_to_index: HashMap<&String, usize> = expanded_nodes.iter().enumerate().map(|(index, node)| (&node.id, index)).collect();

    // map from edge id to connections, such that edge_id is the "from" edge of the connection
    // connection is a hashset to avoid multiple connections from the same edge to the same edge
    let edge_id_to_connections = {
        let mut map: HashMap<&String, HashSet<&Connection>> = HashMap::new();
        for connection in connections {
            map.entry(&connection.from).or_default().insert(connection);
        }
        map
    };

    // edges should be sorted by node index of the tail of the edge
    let mut edges_sorted_by_node_index = Vec::with_capacity(edges.len());
    for edge in edges {
        let &from_node_index = node_id_to_index
            .get(&Node::get_node_id_for_internal_node(&edge.from, &edge.id))
            .expect("From node not found in nodes document root");
        let &to_node_index = node_id_to_index
            .get(&Node::get_node_id_for_internal_node(&edge.to, &edge.id))
            .expect("To node not found in nodes document root");

        let from_node = &expanded_nodes[from_node_index];
        let to_node = &expanded_nodes[to_node_index];

        let length = edge.get_length((from_node.x, from_node.y), (to_node.x, to_node.y));

        // weight is the default weight, which is the length divided by the free-flow speed
        let weight = f64::max(length / (edge.get_speed() * GLOBAL_FREE_FLOW_SPEED_FACTOR), MIN_EDGE_WEIGHT);

        let from_node_index = from_node_index as u32;
        let to_node_index = to_node_index as u32;

        edges_sorted_by_node_index.push(FlattenedSumoEdge::new(
            from_node_index,
            to_node_index,
            edge.id.clone(),
            weight,
            length,
            edge.get_capacity(),
            edge.num_lanes.unwrap_or(1),
            edge.speed.unwrap_or(SUMO_DEFAULT_SPEED),
        ));

        // add internal edges (connections)
        for con in edge_id_to_connections.get(&edge.id).unwrap_or(&HashSet::new()) {
            let node_id = Node::get_node_id_from_internal_node(&to_node.id);

            let internal_from_node_id = Node::get_node_id_for_internal_node(&node_id, &edge.id);
            let &from_node_index = node_id_to_index
                .get(&internal_from_node_id)
                .expect(&format!("Internal Node with id {} not found in list of nodes", internal_from_node_id));

            let internal_to_node_id: String = Node::get_node_id_for_internal_node(&node_id, &con.to);
            let &to_node_index = node_id_to_index
                .get(&internal_to_node_id)
                .expect(&format!("Internal Node with id {} not found in list of nodes", internal_to_node_id));

            let from_node_index = from_node_index as u32;
            let to_node_index = to_node_index as u32;

            edges_sorted_by_node_index.push(FlattenedSumoEdge::new(
                from_node_index,
                to_node_index,
                FlattenedSumoEdge::get_edge_id_for_connection(&edge.id, &con.to),
                MIN_EDGE_WEIGHT,
                0.0,
                f64::MAX, // infinite capacity for internal edges
                1,
                f64::MAX, // infinite speed for internal edges
            ));
        }
    }

    edges_sorted_by_node_index.sort_by_key(|e| (e.from_node_index, e.to_node_index));

    (expanded_nodes, edges_sorted_by_node_index)
}

/// For each edge, we create two nodes with the id "<node_id>\n<edge_id>" with weight MIN_EDGE_WEIGHT.
/// The position should be the same as the original node.
/// This is necessary to model turn restrictions and turn costs.
#[cfg(feature = "expand-sumo-nodes")]
fn expand_nodes(nodes: &Vec<Node>, edges: &Vec<Edge>) -> Vec<Node> {
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

    #[cfg(not(feature = "expand-sumo-nodes"))]
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
                    speed: Some(SUMO_DEFAULT_SPEED),
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
                    speed: Some(SUMO_DEFAULT_SPEED),
                    length: None,
                    lanes: vec![],
                    params: vec![],
                    priority: Some(-1),
                },
            ],
        };

        let (td_graph, _, _) = get_routing_kit_td_graph_from_sumo(&nodes, &edges, Some(0.0), Some(86400.0), Some(86400.0));

        assert_eq!(td_graph.0.len(), 3); // 2 nodes + 1 for the end
        assert_eq!(td_graph.1.len(), 2); // 2 edges
        assert_eq!(td_graph.2.len(), 2 + 1); // 2 edges each having 1 ipp

        // Without node expansion, nodes remain the same
        let actual_node_names: Vec<_> = nodes.nodes.iter().map(|n| n.id.as_str()).collect();
        assert_eq!(actual_node_names, vec!["n2", "n1"]);
    }

    #[cfg(not(feature = "expand-sumo-nodes"))]
    #[test]
    fn test_td_graph_with_some_connections() {
        // 2 nodes, 2 edges (no connections without feature flag)
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
                    speed: Some(SUMO_DEFAULT_SPEED),
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
                    speed: Some(SUMO_DEFAULT_SPEED),
                    length: None,
                    lanes: vec![],
                    params: vec![],
                    priority: Some(-1),
                },
            ],
        };

        let (td_graph, _, edge_ids) = get_routing_kit_td_graph_from_sumo(&nodes, &edges, Some(0.0), Some(86400.0), Some(86400.0));

        // Without expansion: 2 original edges only
        assert_eq!(td_graph.1.len(), 2);

        // No connection edges without the feature
        assert!(!edge_ids.iter().any(|id| id.contains("$$")));

        // Without node expansion, nodes remain the same
        let actual_node_names: Vec<_> = nodes.nodes.iter().map(|n| n.id.as_str()).collect();
        assert_eq!(actual_node_names, vec!["n2", "n1"]);
    }

    #[cfg(not(feature = "expand-sumo-nodes"))]
    #[test]
    fn test_td_graph_with_multi_connections() {
        // 2 nodes, 2 edges (no connections without feature flag)
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
                    speed: Some(SUMO_DEFAULT_SPEED),
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
                    speed: Some(SUMO_DEFAULT_SPEED),
                    length: None,
                    lanes: vec![],
                    params: vec![],
                    priority: Some(-1),
                },
            ],
        };

        let (td_graph, _, edge_ids) = get_routing_kit_td_graph_from_sumo(&nodes, &edges, Some(0.0), Some(86400.0), Some(86400.0));

        // Without expansion: 2 original edges only
        assert_eq!(td_graph.1.len(), 2);

        // No connection edges without the feature
        assert!(!edge_ids.iter().any(|id| id.contains("$$")));

        // Without node expansion, nodes remain the same
        let actual_node_names: Vec<_> = nodes.nodes.iter().map(|n| n.id.as_str()).collect();
        assert_eq!(actual_node_names, vec!["n2", "n1"]);
    }

    // Tests with node expansion feature enabled
    #[cfg(feature = "expand-sumo-nodes")]
    mod expanded_tests {
        use super::*;
        use crate::sumo::connections::{Connection, ConnectionsDocumentRoot};

        #[test]
        fn test_convert_sumo_to_td_graph_with_expansion() {
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
                        speed: Some(SUMO_DEFAULT_SPEED),
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
                        speed: Some(SUMO_DEFAULT_SPEED),
                        length: None,
                        lanes: vec![],
                        params: vec![],
                        priority: Some(-1),
                    },
                ],
            };

            let connections = ConnectionsDocumentRoot { connections: vec![] };

            let (td_graph, _, _, _) = get_routing_kit_td_graph_from_sumo(&nodes, &edges, &connections, None, None, None);

            assert_eq!(td_graph.0.len(), 5); // 4 internal nodes + 1 for the end
            assert_eq!(td_graph.1.len(), 2); // 2 edges
            assert_eq!(td_graph.2.len(), 2 + 1); // 2 edges each having 1 ipp
        }

        #[test]
        fn test_td_graph_with_some_connections_expanded() {
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
                        speed: Some(SUMO_DEFAULT_SPEED),
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
                        speed: Some(SUMO_DEFAULT_SPEED),
                        length: None,
                        lanes: vec![],
                        params: vec![],
                        priority: Some(-1),
                    },
                ],
            };

            let connections = ConnectionsDocumentRoot {
                connections: vec![Connection {
                    from: String::from("e1"),
                    to: String::from("e2"),
                    from_lane: Some(String::from("0")),
                    to_lane: Some(String::from("0")),
                }],
            };

            let (td_graph, _, _, edge_ids) = get_routing_kit_td_graph_from_sumo(&nodes, &edges, &connections, None, None, None);

            // 2 original edges + 1 connection edge
            assert_eq!(td_graph.1.len(), 3);
            // The edge_ids should contain the connection edge id
            assert!(edge_ids
                .iter()
                .any(|id| id.contains(&FlattenedSumoEdge::get_edge_id_for_connection("e1", "e2"))));
        }

        #[test]
        fn test_td_graph_with_multi_connections_expanded() {
            // 2 nodes, 2 edges, connections from e1 to e2 and e2 to e1
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
                        speed: Some(SUMO_DEFAULT_SPEED),
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
                        speed: Some(SUMO_DEFAULT_SPEED),
                        length: None,
                        lanes: vec![],
                        params: vec![],
                        priority: Some(-1),
                    },
                ],
            };

            let connections = ConnectionsDocumentRoot {
                connections: vec![
                    Connection {
                        from: String::from("e1"),
                        to: String::from("e2"),
                        from_lane: Some(String::from("0")),
                        to_lane: Some(String::from("0")),
                    },
                    Connection {
                        from: String::from("e2"),
                        to: String::from("e1"),
                        from_lane: Some(String::from("0")),
                        to_lane: Some(String::from("0")),
                    },
                ],
            };

            let (td_graph, _, _, edge_ids) = get_routing_kit_td_graph_from_sumo(&nodes, &edges, &connections, None, None, None);

            // 2 original edges + 2 connection edges
            assert_eq!(td_graph.1.len(), 4);

            // All possible connection edge ids should be present
            let expected_connections = vec![
                FlattenedSumoEdge::get_edge_id_for_connection("e1", "e2"),
                FlattenedSumoEdge::get_edge_id_for_connection("e2", "e1"),
            ];
            for conn in expected_connections {
                assert!(edge_ids.iter().any(|id| id.contains(&conn)), "Missing connection edge id: {}", conn);
            }
        }
    }

    #[test]
    fn test_get_departure_times_default_values() {
        // Test with all None values - should use defaults
        let result = get_departure_times(None, None, None);

        // Default is from 0 to 86400 (one day) with a single interval
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0);
    }

    #[test]
    fn test_get_departure_times_explicit_single_interval() {
        // Test with explicit values covering the whole time span
        let result = get_departure_times(Some(0.0), Some(86400.0), Some(86400.0));

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0);
    }

    #[test]
    fn test_get_departure_times_multiple_intervals() {
        // Test with multiple intervals throughout the day
        let result = get_departure_times(Some(0.0), Some(3600.0), Some(900.0));

        // From 0 to 3600 with 900s intervals: 0, 900, 1800, 2700
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], 0);
        assert_eq!(result[1], 900);
        assert_eq!(result[2], 1800);
        assert_eq!(result[3], 2700);
    }

    #[test]
    fn test_get_departure_times_custom_begin_and_end() {
        // Test with custom begin and end times (e.g., rush hour)
        let result = get_departure_times(Some(21600.0), Some(25200.0), Some(1800.0));

        // From 21600 (6am) to 25200 (7am) with 1800s (30min) intervals: 21600, 23400
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], 21600);
        assert_eq!(result[1], 23400);
    }

    #[test]
    fn test_get_departure_times_small_intervals() {
        // Test with very small intervals
        let result = get_departure_times(Some(0.0), Some(10.0), Some(2.0));

        // From 0 to 10 with 2s intervals: 0, 2, 4, 6, 8
        assert_eq!(result.len(), 5);
        assert_eq!(result[0], 0);
        assert_eq!(result[1], 2);
        assert_eq!(result[2], 4);
        assert_eq!(result[3], 6);
        assert_eq!(result[4], 8);
    }

    #[test]
    fn test_get_departure_times_interval_larger_than_span() {
        // Test when interval is larger than the time span
        let result = get_departure_times(Some(0.0), Some(100.0), Some(200.0));

        // Should only have one departure time at the beginning
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0);
    }

    #[test]
    fn test_get_departure_times_partial_interval_at_end() {
        // Test when the last interval doesn't complete before end
        let result = get_departure_times(Some(0.0), Some(100.0), Some(30.0));

        // From 0 to 100 with 30s intervals: 0, 30, 60, 90 (100 is excluded)
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], 0);
        assert_eq!(result[1], 30);
        assert_eq!(result[2], 60);
        assert_eq!(result[3], 90);
    }

    #[test]
    fn test_get_departure_times_exact_multiple() {
        // Test when end is exact multiple of interval
        let result = get_departure_times(Some(0.0), Some(100.0), Some(25.0));

        // From 0 to 100 with 25s intervals: 0, 25, 50, 75
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], 0);
        assert_eq!(result[1], 25);
        assert_eq!(result[2], 50);
        assert_eq!(result[3], 75);
    }

    #[test]
    fn test_get_departure_times_non_zero_begin() {
        // Test with non-zero begin time
        let result = get_departure_times(Some(1000.0), Some(3000.0), Some(500.0));

        // From 1000 to 3000 with 500s intervals: 1000, 1500, 2000, 2500
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], 1000);
        assert_eq!(result[1], 1500);
        assert_eq!(result[2], 2000);
        assert_eq!(result[3], 2500);
    }

    #[test]
    fn test_get_departure_times_empty_when_begin_equals_end() {
        // Test edge case where begin equals end
        let result = get_departure_times(Some(100.0), Some(100.0), Some(10.0));

        // Should return empty vector as begin < end is false from the start
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_create_implicit_td_graph_single_departure_time() {
        // Test with a single departure time - each edge should have 1 IPP
        let edges = vec![
            FlattenedSumoEdge::new(0, 1, String::from("e1"), 10.0, 100.0, 1.0, 1, 10.0),
            FlattenedSumoEdge::new(0, 2, String::from("e2"), 20.0, 200.0, 1.0, 1, 10.0),
        ];
        let departure_times = vec![0];

        let (first_out, head, first_ipp_of_arc, ipp_departure_time, ipp_travel_time) = create_implicit_td_graph(3, 2, &edges, &departure_times);

        // Check basic structure
        assert_eq!(first_out.len(), 4); // 3 nodes + 1
        assert_eq!(head.len(), 2); // 2 edges
        assert_eq!(first_ipp_of_arc.len(), 3); // 2 edges + 1

        // Check IPPs: 2 edges * 1 departure time = 2 IPPs
        assert_eq!(ipp_departure_time.len(), 2);
        assert_eq!(ipp_travel_time.len(), 2);

        // Verify IPP content for each edge
        assert_eq!(ipp_departure_time[0], 0);
        assert_eq!(ipp_travel_time[0], 10000); // 10.0 seconds * 1000 = 10000 ms

        assert_eq!(ipp_departure_time[1], 0);
        assert_eq!(ipp_travel_time[1], 20000); // 20.0 seconds * 1000 = 20000 ms
    }

    #[test]
    fn test_create_implicit_td_graph_multiple_departure_times() {
        // Test with multiple departure times - each edge should have multiple IPPs
        let edges = vec![FlattenedSumoEdge::new(0, 1, String::from("e1"), 10.0, 100.0, 1.0, 1, 10.0)];
        let departure_times = vec![0, 100, 200];

        let (first_out, head, first_ipp_of_arc, ipp_departure_time, ipp_travel_time) = create_implicit_td_graph(2, 1, &edges, &departure_times);

        // Check basic structure
        assert_eq!(first_out.len(), 3); // 2 nodes + 1
        assert_eq!(head.len(), 1); // 1 edge
        assert_eq!(first_ipp_of_arc.len(), 2); // 1 edge + 1

        // Check IPPs: 1 edge * 3 departure times = 3 IPPs
        assert_eq!(ipp_departure_time.len(), 3);
        assert_eq!(ipp_travel_time.len(), 3);

        // Verify IPP content: [(0, 10000), (100, 10000), (200, 10000)]
        assert_eq!(ipp_departure_time[0], 0);
        assert_eq!(ipp_travel_time[0], 10000);

        assert_eq!(ipp_departure_time[1], 100);
        assert_eq!(ipp_travel_time[1], 10000);

        assert_eq!(ipp_departure_time[2], 200);
        assert_eq!(ipp_travel_time[2], 10000);
    }

    #[test]
    fn test_create_implicit_td_graph_multiple_edges_multiple_ipps() {
        // Test with multiple edges and multiple departure times
        let edges = vec![
            FlattenedSumoEdge::new(0, 1, String::from("e1"), 10.0, 100.0, 1.0, 1, 10.0),
            FlattenedSumoEdge::new(0, 2, String::from("e2"), 20.0, 200.0, 1.0, 1, 10.0),
            FlattenedSumoEdge::new(1, 2, String::from("e3"), 15.0, 150.0, 1.0, 1, 10.0),
        ];
        let departure_times = vec![0, 100, 200];

        let (first_out, head, first_ipp_of_arc, ipp_departure_time, ipp_travel_time) = create_implicit_td_graph(3, 3, &edges, &departure_times);

        // Check basic structure
        assert_eq!(first_out.len(), 4); // 3 nodes + 1
        assert_eq!(head.len(), 3); // 3 edges
        assert_eq!(first_ipp_of_arc.len(), 4); // 3 edges + 1

        // Check IPPs: 3 edges * 3 departure times = 9 IPPs
        assert_eq!(ipp_departure_time.len(), 9);
        assert_eq!(ipp_travel_time.len(), 9);

        // Verify first edge IPPs: e1 has IPPs at indices 0, 1, 2
        assert_eq!(ipp_departure_time[0], 0);
        assert_eq!(ipp_travel_time[0], 10000);
        assert_eq!(ipp_departure_time[1], 100);
        assert_eq!(ipp_travel_time[1], 10000);
        assert_eq!(ipp_departure_time[2], 200);
        assert_eq!(ipp_travel_time[2], 10000);

        // Verify second edge IPPs: e2 has IPPs at indices 3, 4, 5
        assert_eq!(ipp_departure_time[3], 0);
        assert_eq!(ipp_travel_time[3], 20000);
        assert_eq!(ipp_departure_time[4], 100);
        assert_eq!(ipp_travel_time[4], 20000);
        assert_eq!(ipp_departure_time[5], 200);
        assert_eq!(ipp_travel_time[5], 20000);

        // Verify third edge IPPs: e3 has IPPs at indices 6, 7, 8
        assert_eq!(ipp_departure_time[6], 0);
        assert_eq!(ipp_travel_time[6], 15000);
        assert_eq!(ipp_departure_time[7], 100);
        assert_eq!(ipp_travel_time[7], 15000);
        assert_eq!(ipp_departure_time[8], 200);
        assert_eq!(ipp_travel_time[8], 15000);

        // Verify first_ipp_of_arc points to correct IPP indices
        assert_eq!(first_ipp_of_arc[0], 0); // e1 starts at IPP 0
        assert_eq!(first_ipp_of_arc[1], 3); // e2 starts at IPP 3
        assert_eq!(first_ipp_of_arc[2], 6); // e3 starts at IPP 6
        assert_eq!(first_ipp_of_arc[3], 9); // end marker (total number of IPPs)
    }

    #[test]
    fn test_create_implicit_td_graph_node_without_outgoing_edges() {
        // Test graph where some nodes have no outgoing edges
        let edges = vec![
            FlattenedSumoEdge::new(0, 1, String::from("e1"), 10.0, 100.0, 1.0, 1, 10.0),
            // Node 1 has no outgoing edges
            FlattenedSumoEdge::new(2, 1, String::from("e2"), 20.0, 200.0, 1.0, 1, 10.0),
        ];
        let departure_times = vec![0, 100];

        let (first_out, head, _first_ipp_of_arc, ipp_departure_time, ipp_travel_time) = create_implicit_td_graph(3, 2, &edges, &departure_times);

        // Check basic structure
        assert_eq!(first_out.len(), 4); // 3 nodes + 1
        assert_eq!(head.len(), 2); // 2 edges

        // Node 0 has 1 outgoing edge (to index 0)
        assert_eq!(first_out[0], 0);
        // Node 1 has no outgoing edges (same index as next node)
        assert_eq!(first_out[1], 1);
        // Node 2 has 1 outgoing edge (to index 1)
        assert_eq!(first_out[2], 1);
        // End marker
        assert_eq!(first_out[3], 2);

        // Check IPPs: 2 edges * 2 departure times = 4 IPPs
        assert_eq!(ipp_departure_time.len(), 4);
        assert_eq!(ipp_travel_time.len(), 4);
    }

    #[test]
    fn test_create_implicit_td_graph_empty_graph() {
        // Test with no edges
        let edges = vec![];
        let departure_times = vec![0, 100, 200];

        let (first_out, head, first_ipp_of_arc, ipp_departure_time, ipp_travel_time) = create_implicit_td_graph(2, 0, &edges, &departure_times);

        // Check basic structure
        assert_eq!(first_out.len(), 3); // 2 nodes + 1
        assert_eq!(head.len(), 0); // no edges
        assert_eq!(first_ipp_of_arc.len(), 1); // 0 edges + 1

        // No edges means no IPPs
        assert_eq!(ipp_departure_time.len(), 0);
        assert_eq!(ipp_travel_time.len(), 0);

        // All nodes point to index 0 (no edges)
        assert_eq!(first_out[0], 0);
        assert_eq!(first_out[1], 0);
        assert_eq!(first_out[2], 0);

        assert_eq!(first_ipp_of_arc[0], 0);
    }

    #[test]
    fn test_create_implicit_td_graph_ipp_indexing() {
        // Test to verify correct IPP indexing with varying departure times
        let edges = vec![
            FlattenedSumoEdge::new(0, 1, String::from("e1"), 5.0, 50.0, 1.0, 1, 10.0),
            FlattenedSumoEdge::new(1, 2, String::from("e2"), 10.0, 100.0, 1.0, 1, 10.0),
        ];
        let departure_times = vec![0, 300, 600, 900, 1200];

        let (_first_out, _head, first_ipp_of_arc, ipp_departure_time, ipp_travel_time) = create_implicit_td_graph(3, 2, &edges, &departure_times);

        // Check IPPs: 2 edges * 5 departure times = 10 IPPs
        assert_eq!(ipp_departure_time.len(), 10);
        assert_eq!(ipp_travel_time.len(), 10);

        // Verify first_ipp_of_arc indices
        assert_eq!(first_ipp_of_arc[0], 0); // e1 starts at IPP 0
        assert_eq!(first_ipp_of_arc[1], 5); // e2 starts at IPP 5 (after 5 IPPs of e1)
        assert_eq!(first_ipp_of_arc[2], 10); // end marker (total number of IPPs)

        // Verify e1 IPPs (indices 0-4)
        for i in 0..5 {
            assert_eq!(ipp_departure_time[i], departure_times[i]);
            assert_eq!(ipp_travel_time[i], 5000); // 5.0 * 1000
        }

        // Verify e2 IPPs (indices 5-9)
        for i in 0..5 {
            assert_eq!(ipp_departure_time[5 + i], departure_times[i]);
            assert_eq!(ipp_travel_time[5 + i], 10000); // 10.0 * 1000
        }
    }
}
