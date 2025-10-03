use serde_derive::Deserialize;

use crate::sumo::{SumoTimestamp, SumoTravelTime};

/// this is read from `dump_*.xml` files
#[derive(Debug, Deserialize)]
#[serde(rename = "meandata")]
pub struct MeandataDocumentRoot {
    #[serde(rename = "interval")]
    pub intervals: Vec<Interval>,
}

#[derive(Debug, Deserialize)]
pub struct Interval {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@begin")]
    pub begin: SumoTimestamp,
    #[serde(rename = "@end")]
    pub end: SumoTimestamp,

    #[serde(rename = "edge", default)]
    pub edges: Vec<Edge>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Edge {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@traveltime")]
    pub traveltime: Option<SumoTravelTime>,
    #[serde(rename = "@density")]
    pub density: Option<f64>,
    #[serde(rename = "@speed")]
    pub speed: Option<f64>,
    #[serde(rename = "@sampledSeconds")]
    pub sampled_seconds: Option<f64>,
}

impl Edge {
    pub fn get_average_traffic_volume(&self) -> f64 {
        self.density.unwrap_or(0.0) * self.speed.unwrap_or(0.0) * 3.6
    }
}
