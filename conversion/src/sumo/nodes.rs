use serde_derive::Deserialize;
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
    pub x: f64,
    #[serde(rename = "@y")]
    pub y: f64,
}
