use std::{env, path::Path};

use clap::Parser;
use conversion::sumo::{
    CON_XML, EDG_XML, NOD_XML, XmlReader, XmlWriter,
    connections::{Connection, ConnectionsDocumentRoot},
    connections_writer::SumoConnectionsWriter,
    edges_reader::SumoEdgesReader,
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
    let nodes_document_root = SumoNodesReader::read(&nod_file).expect("Failed to read nodes from file");

    let mut connection_document_root = ConnectionsDocumentRoot { connections: vec![] };

    // for all nodes, create a connection from all incoming edges to all its outgoing edges
    for node in nodes_document_root.nodes {
        for out_edge_of_node in edges_document_root.edges.iter().filter(|e| e.from == node.id) {
            for in_edge_of_node in edges_document_root.edges.iter().filter(|e| e.to == node.id) {
                if (in_edge_of_node.from != out_edge_of_node.to) && (in_edge_of_node.id == out_edge_of_node.id) {
                    // Skip self-connections and connections not going over the same node
                    continue;
                }
                // Create a connection from incoming edge to outgoing edge
                let connection = Connection {
                    from: in_edge_of_node.id.clone(),
                    to: out_edge_of_node.id.clone(),
                    from_lane: Some(String::from("0")), // assuming lane 0 for simplicity
                    to_lane: Some(String::from("0")),   // assuming lane 0 for simplicity
                };
                connection_document_root.connections.push(connection);
            }
        }
    }

    // Write the connections to the con.xml file
    SumoConnectionsWriter::write(&con_file, &connection_document_root).unwrap();
}

/// Command-line arguments for creating a complete connections file
#[derive(Parser, Debug)]
#[command(version, about = "Sumo Connection File Creator options", long_about = None)]
pub struct Args {
    /// the directory containing the input files
    #[arg(long = "input-dir", default_value_t = String::from(env::current_dir().unwrap().to_str().unwrap()))]
    pub input_dir: String,

    /// the files `<input-prefix>.con.xml`, `<input-prefix>.nod.xml`, `<input-prefix>.edg.xml` will be read as input
    #[arg(long = "input-prefix", default_value = "")]
    pub input_prefix: String,
}
