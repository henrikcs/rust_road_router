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
    pub speed: Option<f64>,

    /// if length is not set, the length will be the euclidean distance between `from` and `to`
    #[serde(rename = "@length")]
    pub length: Option<f64>,

    #[serde(rename = "lane", default)]
    pub lanes: Vec<Lane>,

    #[serde(rename = "param", default)]
    pub params: Vec<Param>,
    // TODO: support splits?
}

impl Edge {
    pub fn get_speed(&self) -> f64 {
        self.speed.unwrap_or(13.9) // default speed is 13.9 m/s (50 km/h)
    }

    pub fn get_length(&self, (from_x, from_y): (f64, f64), (to_x, to_y): (f64, f64)) -> f64 {
        self.length.unwrap_or_else(|| {
            let dx = from_x - to_x;
            let dy = from_y - to_y;
            (dx * dx + dy * dy).sqrt() // euclidean distance
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct Lane {
    #[serde(rename = "@index")]
    pub index: u32,

    #[serde(rename = "param", default)]
    pub params: Vec<Param>,
}
