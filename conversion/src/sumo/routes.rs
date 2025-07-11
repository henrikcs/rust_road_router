use serde_derive::{Deserialize, Serialize};

use crate::sumo::{SumoTimestamp, SumoTravelTime};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "routes")]
pub struct RoutesDocumentRoot {
    #[serde(rename = "vehicle")]
    pub vehicles: Vec<Vehicle>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vehicle {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@depart")]
    pub depart: SumoTimestamp,
    #[serde(default, rename = "@departLane")]
    pub depart_lane: Option<String>,
    #[serde(default, rename = "@departPos")]
    pub depart_pos: Option<String>,
    #[serde(default, rename = "@departSpeed")]
    pub depart_speed: Option<String>,
    #[serde(default, rename = "route")]
    pub route: Option<Route>,
    #[serde(default, rename = "routeDistribution")]
    pub route_distribution: Option<RouteDistribution>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Route {
    #[serde(default, rename = "@edges")]
    pub edges: String,
    #[serde(default, rename = "@cost")]
    pub cost: Option<SumoTravelTime>,
    #[serde(default, rename = "@probability")]
    pub probability: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteDistribution {
    #[serde(default, rename = "@last")]
    pub last: u32,
    #[serde(default, rename = "route")]
    pub routes: Vec<Route>,
}
