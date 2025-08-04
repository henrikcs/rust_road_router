use serde_derive::Deserialize;

use crate::sumo::SumoPosition;

#[derive(Debug, Deserialize, Default)]
#[serde(rename = "nodes")]
pub struct NodesDocumentRoot {
    #[serde(rename = "node", default)]
    pub nodes: Vec<Node>,

    #[serde(rename = "location", default)]
    pub location: Option<Location>,
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

/// Example XML node:
/// <location netOffset="1008027.0048680,-4394297.4136840" convBoundary="0.0000000,0.0000000,3169642.5096950,2468532.6478150" origBoundary="-8.515209,38.871680,31.755280,60.945676" projParameter="+proj=utm +zone=32 +ellps=WGS84 +datum=WGS84 +units=m +no_defs"/>
#[derive(Debug, Deserialize)]
pub struct Location {
    #[serde(rename = "@netOffset")]
    pub net_offset: String,
    #[serde(rename = "@convBoundary")]
    pub conv_boundary: String,
    #[serde(rename = "@origBoundary")]
    pub orig_boundary: String,
    #[serde(rename = "@projParameter")]
    pub proj_parameter: String,
}
