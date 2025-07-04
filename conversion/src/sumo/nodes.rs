use serde_derive::Deserialize;
#[derive(Debug, Deserialize)]
#[serde(rename = "nodes")]
pub struct NodesDocumentRoot {
    #[serde(rename = "node", default)]
    pub nodes: Vec<Node>,
}

impl NodesDocumentRoot {
    pub fn get_latitude_longitude(&self) -> (Vec<f32>, Vec<f32>) {
        self.nodes.iter().map(|node| (node.x, node.y)).collect()
    }
}

#[derive(Debug, Deserialize)]
pub struct Node {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@x")]
    pub x: f32,
    #[serde(rename = "@y")]
    pub y: f32,
}
