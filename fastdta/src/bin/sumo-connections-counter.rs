use std::{collections::HashMap, env, path::Path};

use clap::Parser;
use conversion::sumo::{
    CON_XML, EDG_XML, NOD_XML, XmlReader, connections_reader::SumoConnectionsReader, edges::Edge, edges_reader::SumoEdgesReader, nodes_reader::SumoNodesReader,
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
    let nodes_document_root = SumoNodesReader::read(&nod_file).expect("Failed to read nodes from file");
    let connections_document_root = SumoConnectionsReader::read(&con_file).expect("Failed to read connections from file");

    let mut number_of_incomplete_nodes: u32 = 0;

    let mut out_edges_by_node: HashMap<&String, Vec<&String>> = HashMap::new();
    let mut in_edges_by_node: HashMap<&String, Vec<&String>> = HashMap::new();
    let mut edge_by_edge_id: HashMap<&String, &Edge> = HashMap::new();

    // preprocess the vectors of outgoing and incoming edges by node
    for edge in edges_document_root.edges.iter() {
        edge_by_edge_id.insert(&edge.id, &edge);
        if let Some(node_index) = nodes_document_root.nodes.iter().position(|n| n.id == edge.from) {
            out_edges_by_node.entry(&nodes_document_root.nodes[node_index].id).or_default().push(&edge.id);
        }
        if let Some(node_index) = nodes_document_root.nodes.iter().position(|n| n.id == edge.to) {
            in_edges_by_node.entry(&nodes_document_root.nodes[node_index].id).or_default().push(&edge.id);
        }
    }

    for node in nodes_document_root.nodes.iter() {
        // connections.from and connection.to is an edge id
        // number of outgoing edges * number of incoming edges should match the number of connections
        // number of connections with .from. == node.id should match the number of edges with .from == node.id
        let number_of_outgoing_edges = out_edges_by_node.get(&node.id).unwrap_or(&vec![]).len();
        let number_of_incoming_edges = in_edges_by_node.get(&node.id).unwrap_or(&vec![]).len();
        let number_of_connections = connections_document_root
            .connections
            .iter()
            .filter(|c| edge_by_edge_id.get(&c.from).unwrap().to == node.id || edge_by_edge_id.get(&c.to).unwrap().from == node.id)
            .count();

        if number_of_outgoing_edges * number_of_incoming_edges != number_of_connections {
            number_of_incomplete_nodes += 1;
        }
    }

    println!("Number of incomplete nodes: {}", &number_of_incomplete_nodes);
    println!("Number of nodes: {}", &nodes_document_root.nodes.len());
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
