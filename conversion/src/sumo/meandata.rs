use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

use crate::sumo::{SumoTimestamp, SumoTravelTime};

/// this is read from `dump_*.xml` files
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename = "meandata")]
pub struct MeandataDocumentRoot {
    #[serde(rename = "interval")]
    pub intervals: Vec<Interval>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
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

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Edge {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@traveltime")]
    pub traveltime: Option<SumoTravelTime>,
    #[serde(rename = "@overlapTraveltime")]
    pub overlap_traveltime: Option<SumoTravelTime>,
    #[serde(rename = "@density")]
    pub density: Option<f64>,
    #[serde(rename = "@speed")]
    pub speed: Option<f64>,
    #[serde(rename = "@sampledSeconds")]
    pub sampled_seconds: Option<f64>,
    #[serde(rename = "@laneDensity")]
    pub lane_density: Option<f64>,
    #[serde(rename = "@departed")]
    pub departed: Option<u32>,
    #[serde(rename = "@arrived")]
    pub arrived: Option<u32>,
    #[serde(rename = "@entered")]
    pub entered: Option<u32>,
    #[serde(rename = "@left")]
    pub left: Option<u32>,

    #[serde(skip_deserializing)]
    pub dbg_entered: u32,
    #[serde(skip_deserializing)]
    pub dbg_left: u32,
}

impl Edge {
    pub fn get_density(&self, period: f64, length: f64) -> f64 {
        if self.sampled_seconds.is_none() || self.overlap_traveltime.is_none() {
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

    pub fn add_edge(&mut self, edge: Edge) {
        let edge_id = edge.id.clone();
        self.edges.push(edge);

        if self.edge_map.is_none() {
            self.init_edge_map();
        } else {
            let map = self.edge_map.as_mut().unwrap();
            map.insert(edge_id, self.edges.len() - 1);
        }
    }

    pub fn get_edge(&mut self, id: &str) -> Option<&Edge> {
        if self.edge_map.is_none() {
            self.init_edge_map();
        }
        self.edge_map.as_ref().unwrap().get(id).map(|&i| &self.edges[i])
    }

    pub fn get_edge_mut(&mut self, id: &str) -> Option<&mut Edge> {
        if self.edge_map.is_none() {
            self.init_edge_map();
        }
        self.edge_map.as_ref().unwrap().get(id).map(|&i| &mut self.edges[i])
    }

    fn init_edge_map(&mut self) {
        let map: HashMap<String, usize> = self.edges.iter().enumerate().map(|(i, e)| (e.id.clone(), i)).collect();
        self.edge_map = Some(map);
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
