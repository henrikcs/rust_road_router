use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ConnectionsDocumentRoot {
    #[serde(rename = "connection", default)]
    pub connections: Vec<Connection>,
}

/// usally: <connection from="A1A2" to="A2B2" fromLane="0" toLane="0">
#[derive(Debug, Deserialize, Serialize)]
pub struct Connection {
    #[serde(rename = "@from")]
    pub from: String,

    #[serde(rename = "@to")]
    pub to: String,

    #[serde(rename = "@fromLane")]
    pub from_lane: Option<String>,

    #[serde(rename = "@toLane")]
    pub to_lane: Option<String>,
}
