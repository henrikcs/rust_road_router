use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

use crate::{
    sumo::{SumoTimestamp, SumoTravelTime},
    SUMO_MAX_TRAVEL_TIME,
};

/// this is read from `dump_*.xml` files
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "meandata")]
pub struct MeandataDocumentRoot {
    #[serde(rename = "interval")]
    pub intervals: Vec<Interval>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Interval {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@begin")]
    pub begin: SumoTimestamp,
    #[serde(rename = "@end")]
    pub end: SumoTimestamp,

    #[serde(rename = "edge", default)]
    pub edges: Vec<Edge>,

    #[serde(skip)]
    edge_map: Option<HashMap<String, usize>>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
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
    #[serde(rename = "@laneDensity")]
    pub lane_density: Option<f64>,
}

impl Edge {
    // pub fn get_traffic_volume(&self, period: f64) -> f64 {
    //     if self.traveltime.is_none() || self.sampled_seconds.is_none() {
    //         return 0.0;
    //     }
    //     if self.traveltime == Some(0.0) || self.sampled_seconds == Some(0.0) {
    //         return 0.0;
    //     }

    //     3600.0 * self.sampled_seconds.unwrap() / (period * self.traveltime.unwrap())
    // }

    pub fn get_density(&self, period: f64, length: f64) -> f64 {
        if self.sampled_seconds.is_none() || self.traveltime.is_none() {
            return 0.0;
        }
        if self.sampled_seconds == Some(0.0) || length == 0.0 {
            return 0.0;
        }
        1000.0 * self.sampled_seconds.unwrap() / (period * length) as f64
    }

    pub fn get_lane_density(&self, period: f64, length: f64, lanes: u32) -> f64 {
        self.get_density(period, length) / lanes as f64
    }

    // pub fn get_estimated_travel_time(&self, period: f64, length: f64, lanes: u32, free_flow_tt: f64) -> f64 {
    //     // try out greenshields model:
    //     let free_flow_speed = length / free_flow_tt; // in m/s
    //     let jam_density = 133.33; // vehicles per km per lane
    //     let density = self.get_density(period, length) / lanes as f64;

    //     let velocity = free_flow_speed * (1.0 - density / jam_density);

    //     if velocity <= 0.0 {
    //         return SUMO_MAX_TRAVEL_TIME;
    //     }
    //     // in m/s
    //     f64::max(length / velocity, free_flow_tt) // in seconds
    // }
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

impl MeandataDocumentRoot {
    pub fn empty() -> Self {
        MeandataDocumentRoot {
            intervals: vec![Interval {
                id: "".to_string(),
                begin: 0.0,
                end: 86400.0,
                edges: vec![],
                edge_map: None,
            }],
        }
    }
}
