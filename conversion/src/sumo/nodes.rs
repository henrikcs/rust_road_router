use serde_derive::Deserialize;

use crate::sumo::SumoPosition;

#[derive(Debug, Deserialize)]
#[serde(rename = "nodes")]
pub struct NodesDocumentRoot {
    #[serde(rename = "node", default)]
    pub nodes: Vec<Node>,
}

#[derive(Debug, Deserialize)]
pub struct Node {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@x")]
    pub x: SumoPosition,
    #[serde(rename = "@y")]
    pub y: SumoPosition,
}
