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

    pub fn get_traffic_volume(&self, period: u32) -> f64 {
        if self.traveltime.is_none() || self.sampled_seconds.is_none() {
            return 0.0;
        }
        if self.traveltime == Some(0.0) || self.sampled_seconds == Some(0.0) {
            return 0.0;
        }

        self.sampled_seconds.unwrap() / (period as f64 * self.traveltime.unwrap()) * 3600.0
    }

    pub fn get_length(&self, period: u32) -> f64 {
        if self.density == Some(0.0) || self.speed == Some(0.0) {
            return 0.0;
        }
        self.sampled_seconds.unwrap_or(0.0) / (period as f64) * 1000.0 / self.density.unwrap() as f64
    }
}
