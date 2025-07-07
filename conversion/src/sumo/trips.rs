use serde_derive::Deserialize;

use crate::sumo::SumoTimestamp;

#[derive(Debug, Deserialize)]
#[serde(rename = "routes")]
pub struct TripsDocumentRoot {
    #[serde(rename = "trip", default)]
    pub vehicles: Vec<Trip>,
}

#[derive(Debug, Deserialize)]
pub struct Trip {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@from")]
    pub from: String,
    #[serde(rename = "@to")]
    pub to: String,
    #[serde(rename = "@depart")]
    pub depart: SumoTimestamp,
    #[serde(default, rename = "@departLane")]
    pub depart_lane: Option<String>,
    #[serde(default, rename = "@departPos")]
    pub depart_pos: Option<String>,
    #[serde(default, rename = "@departSpeed")]
    pub depart_speed: Option<String>,
}
