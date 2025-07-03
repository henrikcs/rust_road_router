use serde_derive::Deserialize;

use crate::sumo::base_types::Param;

#[derive(Debug, Deserialize)]
pub struct EdgesDocumentRoot {
    #[serde(rename = "edge", default)]
    pub edges: Vec<Edge>,
}

#[derive(Debug, Deserialize)]
pub struct Edge {
    #[serde(rename = "@id")]
    pub id: String,

    #[serde(rename = "@from")]
    pub from: String,

    #[serde(rename = "@to")]
    pub to: String,

    /// if num_lanes is not set, it is 1
    #[serde(rename = "@numLanes")]
    pub num_lanes: Option<u32>,

    /// if speed is not set, it's 13.9 m/s (50km/h)
    #[serde(rename = "@speed")]
    pub speed: Option<f32>,

    /// if length is not set, the length will be the euclidean distance between `from` and `to`
    #[serde(rename = "@length")]
    pub length: Option<f32>,

    #[serde(rename = "lane", default)]
    pub lanes: Vec<Lane>,

    #[serde(rename = "param", default)]
    pub params: Vec<Param>,
    // TODO: support splits?
}

#[derive(Debug, Deserialize)]
pub struct Lane {
    #[serde(rename = "@index")]
    pub index: u32,

    #[serde(rename = "param", default)]
    pub params: Vec<Param>,
}
