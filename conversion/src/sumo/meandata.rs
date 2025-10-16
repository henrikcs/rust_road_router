use std::collections::HashMap;

use serde_derive::Deserialize;

use crate::sumo::{SumoTimestamp, SumoTravelTime};

/// this is read from `dump_*.xml` files
#[derive(Debug, Deserialize)]
#[serde(rename = "meandata")]
pub struct MeandataDocumentRoot {
    #[serde(rename = "interval")]
    pub intervals: Vec<Interval>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Interval {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@begin")]
    pub begin: SumoTimestamp,
    #[serde(rename = "@end")]
    pub end: SumoTimestamp,

    #[serde(rename = "edge", default)]
    pub edges: Vec<Edge>,

    edge_map: Option<HashMap<String, usize>>,
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
    pub fn get_traffic_volume(&self, period: f64) -> f64 {
        if self.traveltime.is_none() || self.sampled_seconds.is_none() {
            return 0.0;
        }
        if self.traveltime == Some(0.0) || self.sampled_seconds == Some(0.0) {
            return 0.0;
        }

        self.sampled_seconds.unwrap() / (period * self.traveltime.unwrap()) * 3600.0
    }

    pub fn get_length(&self, period: f64) -> f64 {
        if self.density == Some(0.0) || self.speed == Some(0.0) {
            return 0.0;
        }
        self.sampled_seconds.unwrap_or(0.0) / period * 1000.0 / self.density.unwrap() as f64
    }
}

impl Interval {
    pub fn create(id: String, begin: SumoTimestamp, end: SumoTimestamp, edges: Vec<Edge>) -> Self {
        Interval {
            id,
            begin,
            end,
            edges,
            edge_map: None,
        }
    }

    pub fn get_edge(&mut self, id: &str) -> Option<&mut Edge> {
        if self.edge_map.is_none() {
            let map: HashMap<String, usize> = self.edges.iter().enumerate().map(|(i, e)| (e.id.clone(), i)).collect();
            self.edge_map = Some(map);
        }
        self.edge_map.as_ref().unwrap().get(id).map(|&i| &mut self.edges[i])
    }
}
