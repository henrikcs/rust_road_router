use serde_derive::{Deserialize, Serialize};

use crate::sumo::SumoTimestamp;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "routes")]
pub struct TripsDocumentRoot {
    #[serde(rename = "trip", default)]
    pub trips: Vec<Trip>,
}

#[derive(Debug, Deserialize, Serialize)]
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

/// Struct for reading a MATSim CSV trip file containing the following headers:
/// tripId, legId, tripBeginTime, locationFrom, locationTo
///
#[derive(Debug, Deserialize)]
pub struct MatsimCsvTrip {
    #[serde(rename = "tripId")]
    pub trip_id: String,
    #[serde(rename = "legId")]
    pub leg_id: String,

    /// string is in the format HH:MM:SS
    #[serde(rename = "tripBeginTime")]
    pub trip_begin_time: String,

    /// location is a string in the format: "(<lat>,<lon>: <edge_id>, <edge_start_pos>)"
    #[serde(rename = "locationFrom")]
    pub location_from: String,

    /// location is a string in the format: "(<lat>,<lon>: <edge_id>, <edge_start_pos>)"
    #[serde(rename = "locationTo")]
    pub location_to: String,
}

impl MatsimCsvTrip {
    /// Converts a MatsimCsvTrip to a Sumo Trip
    /// The translated xml would look like:
    /// <trip id="<tripId>-<legId>" depart="<convert_to_seconds_since_midnight(<tripBeginTime>)>" from="<parse_location(<locationFrom>)>" to="<parse_location(<locationTo>)>"
    /// departLane="best" departSpeed="max" departPos="base" arrivalPos="0.0"/>
    pub fn to_sumo_trip(&self) -> Trip {
        Trip {
            id: format!("{}-{}", self.trip_id, self.leg_id),
            from: MatsimCsvTrip::parse_location(&self.location_from),
            to: MatsimCsvTrip::parse_location(&self.location_to),
            depart: MatsimCsvTrip::parse_time(&self.trip_begin_time),
            depart_lane: Some("best".to_string()),
            depart_pos: Some("base".to_string()),
            depart_speed: Some("max".to_string()),
        }
    }

    fn parse_time(hhmmss: &String) -> f64 {
        let parts: Vec<&str> = hhmmss.split(':').collect();
        if parts.len() != 3 {
            panic!("Invalid time format: {}", hhmmss);
        }
        let hours: u32 = parts[0].parse().unwrap_or(0);
        let minutes: u32 = parts[1].parse().unwrap_or(0);
        let seconds: u32 = parts[2].parse().unwrap_or(0);
        (hours * 3600 + minutes * 60 + seconds) as f64
    }

    /// Parses a location string in the format "(<lat>,<lon>: <edge_id>, <edge_start_pos>)"
    /// and returns the edge_id as a String.
    /// If the format is invalid, it panics.
    ///
    /// Example:
    /// "(7.935473010078169,48.871455464594085: -52839, 0.07573593075766594)" would return "-52839"
    fn parse_location(location: &String) -> String {
        let parts: Vec<&str> = location.split(':').collect();
        if parts.len() != 2 {
            panic!("Invalid location format: {}", location);
        }
        let edge_part = parts[1].trim();
        let edge_id = edge_part.split(',').next().unwrap_or("").trim();
        edge_id.to_string()
    }
}
