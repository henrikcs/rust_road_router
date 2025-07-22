use std::{
    collections::{HashMap, HashSet},
    env,
    path::Path,
};

use clap::Parser;
use conversion::sumo::{
    CON_XML, EDG_XML, NOD_XML, XmlReader, connections::Connection, connections_reader::SumoConnectionsReader, edges::Edge, edges_reader::SumoEdgesReader,
    nodes_reader::SumoNodesReader,
};

fn main() {
    let args = Args::parse();

    // Print the input directory
    println!("Input directory: {}", args.input_dir);

    // Here you would typically call your preprocessing function with the input directory
    // preprocess(&args.input_dir);

    let dir = Path::new(&args.input_dir);
    let input_prefix = args.input_prefix;

    let con_file = dir.join(input_prefix.clone() + CON_XML);
    let edg_file = dir.join(input_prefix.clone() + EDG_XML);
    let nod_file = dir.join(input_prefix.clone() + NOD_XML);

    // read all edges with sumoedgesreader
    let edges_document_root = SumoEdgesReader::read(&edg_file).expect("Failed to read edges from file");
    println!("Edges read from file: {}", edges_document_root.edges.len());
    let nodes_document_root = SumoNodesReader::read(&nod_file).expect("Failed to read nodes from file");
    println!("Nodes read from file: {}", nodes_document_root.nodes.len());
    let connections_document_root = SumoConnectionsReader::read(&con_file).expect("Failed to read connections from file");
    println!("Connections read from file: {}", connections_document_root.connections.len());

    let mut out_edges_by_node: HashMap<&String, Vec<&String>> = HashMap::new();
    let mut in_edges_by_node: HashMap<&String, Vec<&String>> = HashMap::new();
    let mut edge_by_edge_id: HashMap<&String, &Edge> = HashMap::new();

    // preprocess the vectors of outgoing and incoming edges by node
    for edge in edges_document_root.edges.iter() {
        edge_by_edge_id.insert(&edge.id, &edge);
        out_edges_by_node.entry(&edge.from).or_default().push(&edge.id);
        in_edges_by_node.entry(&edge.to).or_default().push(&edge.id);
    }

    println!("Preprocessed edges");

    let mut connections_by_from_edge: HashMap<&String, HashSet<&Connection>> = HashMap::new();
    // preprocess the connections by node
    for connection in connections_document_root.connections.iter() {
        connections_by_from_edge.entry(&connection.from).or_default().insert(&connection);
    }

    println!("Preprocessed connections");

    // number of possible connections and actual connections by node
    let mut connection_numbers_by_node: HashMap<&String, (u32, u32)> = HashMap::new();

    for edge in edges_document_root.edges.iter() {
        // connections.from and connection.to is an edge id
        // number of outgoing edges * number of incoming edges should match the number of connections
        // number of connections with .from. == node.id should match the number of edges with .from == node.id
        let number_of_outgoing_edges: u32 = out_edges_by_node.get(&edge.to).map_or(0, |edges| edges.len() as u32);
        let number_of_connections: u32 = connections_by_from_edge.get(&edge.id).map_or(0, |connections| connections.len() as u32);

        // let number_of_connections = connections_document_root
        // .connections
        // .iter()
        // .filter(|c| edge_by_edge_id.get(&c.from).unwrap().to == edge.to || edge_by_edge_id.get(&c.to).unwrap().from == edge.from)
        // .count();

        let current_connections: &(u32, u32) = connection_numbers_by_node.get(&edge.to).unwrap_or(&(0, 0));

        connection_numbers_by_node.insert(
            &edge.to,
            (current_connections.0 + number_of_outgoing_edges, current_connections.1 + number_of_connections),
        );
    }

    let mut missing_connections_by_size: HashMap<i32, u32> = HashMap::new();

    for node in nodes_document_root.nodes.iter() {
        let (expected, actual) = connection_numbers_by_node.get(&node.id).unwrap_or(&(0, 0));

        // println!("Node: {}, Expected: {}, Actual: {}", &node.id, expected, actual);

        missing_connections_by_size.insert(
            *expected as i32 - *actual as i32,
            missing_connections_by_size.get(&(*expected as i32 - *actual as i32)).unwrap_or(&0) + 1,
        );
    }

    // sort by key
    let mut keys: Vec<&i32> = missing_connections_by_size.keys().collect();
    keys.sort();

    for key in keys {
        let count = missing_connections_by_size.get(key).unwrap();
        println!("{} Nodes having {} missing connections", count, key);
    }
}

/// Command-line arguments for counting connections and whether they are complete or not
#[derive(Parser, Debug)]
#[command(version, about = "Sumo Connection Counter options", long_about = None)]
pub struct Args {
    /// the directory containing the input files
    #[arg(long = "input-dir", default_value_t = String::from(env::current_dir().unwrap().to_str().unwrap()))]
    pub input_dir: String,

    /// the files `<input-prefix>.con.xml`, `<input-prefix>.nod.xml`, `<input-prefix>.edg.xml` will be read as input
    #[arg(long = "input-prefix", default_value = "")]
    pub input_prefix: String,
}

/*
connections:

<connection from="-310512" to="200775" fromLane="0" toLane="0"/>
<connection from="-310512" to="200775" fromLane="1" toLane="1"/>
<connection from="-310512" to="200775" fromLane="2" toLane="2"/>
<connection from="-310512" to="200775" fromLane="3" toLane="3"/>
<connection from="-310512" to="200775" fromLane="3" toLane="4"/>
<connection from="-310512" to="200775" fromLane="3" toLane="5"/>
<connection from="-310512" to="3883" fromLane="4" toLane="0"/>
<connection from="-310512" to="3883" fromLane="4" toLane="1"/>
<connection from="-310512" to="CNodeId(rawValue=3668)ZZoneId(rawValue=223)-D" fromLane="5" toLane="2"/>
<connection from="-310512" to="CNodeId(rawValue=3668)ZZoneId(rawValue=223)-D" fromLane="5" toLane="3"/>
<connection from="-310512" to="310512" fromLane="5" toLane="5"/>

 */

/*

<node id="3668" x="16.91" y="10.14" type="priority"/>

// incoming edges:
<edge id="-310512" from="267762" to="3668" priority="-1" numLanes="6" speed="13.89" shape="16.91,10.14 17.01,10.24" allow="passenger">
    <param key="capacity" value="2000.00"/>
</edge>
<edge id="-200775" from="201166" to="3668" priority="-1" numLanes="6" speed="13.89" shape="16.91,10.14 17.01,10.24" allow="passenger">
    <param key="capacity" value="2000.00"/>
</edge>
<edge id="CNodeId(rawValue=3668)ZZoneId(rawValue=223)-O" from="Z:223-OUT" to="3668" priority="-1" numLanes="4" speed="13.89" shape="17.01,10.24 16.91,10.14" allow="passenger">
    <param key="capacity" value="999999.00"/>
</edge>

// outgoing edges:
<edge id="200775" from="3668" to="201166" priority="-1" numLanes="6" speed="13.89" shape="17.01,10.24 16.91,10.14" allow="passenger">
    <param key="capacity" value="2000.00"/>
</edge>
<edge id="3883" from="3668" to="266375" priority="-1" numLanes="6" speed="8.33" shape="17.01,10.24 16.91,10.14" allow="passenger">
    <param key="capacity" value="200.00"/>
</edge>
<edge id="CNodeId(rawValue=3668)ZZoneId(rawValue=223)-D" from="3668" to="Z:223-IN" priority="-1" numLanes="4" speed="13.89" shape="16.91,10.14 17.01,10.24" allow="passenger">
    <param key="capacity" value="999999.00"/>
</edge>
<edge id="310512" from="3668" to="267762" priority="-1" numLanes="6" speed="13.89" shape="17.01,10.24 16.91,10.14" allow="passenger">
    <param key="capacity" value="2000.00"/>
</edge>

*/
