use serde_derive::Deserialize;

use crate::sumo::{SumoPosition, SumoTravelTime};

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
    pub speed: Option<SumoTravelTime>,

    /// if length is not set, the length will be the euclidean distance between `from` and `to`
    #[serde(rename = "@length")]
    pub length: Option<SumoTravelTime>,

    /// priority of a piece of road, used to determine capacity
    #[serde(rename = "@priority")]
    pub priority: Option<i32>,

    #[serde(rename = "lane", default)]
    pub lanes: Vec<Lane>,

    #[serde(rename = "param", default)]
    pub params: Vec<Param>,
}

impl Edge {
    pub fn get_speed(&self) -> SumoTravelTime {
        self.speed.unwrap_or(13.9) // default speed is 13.9 m/s (50 km/h)
    }

    pub fn get_length(&self, (from_x, from_y): (SumoPosition, SumoPosition), (to_x, to_y): (SumoPosition, SumoPosition)) -> SumoTravelTime {
        self.length.unwrap_or_else(|| {
            let dx = from_x - to_x;
            let dy = from_y - to_y;
            (dx * dx + dy * dy).sqrt() // euclidean distance
        })
    }

    /// returns the capacity of the edge in vehicles per hour
    /// based on the formula from https://sumo.dlr.de/docs/Simulation/RoadCapacity.html
    pub fn get_capacity(&self) -> f64 {
        if self.num_lanes == Some(0) {
            return 0.0;
        }

        // if there is a param with key "capacity" use that value
        for param in &self.params {
            if param.key == "capacity" {
                if let Some(value) = &param.value {
                    if let Ok(capacity) = value.parse::<f64>() {
                        return capacity;
                    }
                }
            }
        }

        // Convert priority to road class (negative priority in SUMO)
        let road_class = if let Some(prio) = self.priority { -prio } else { -1 };
        let speed = if let Some(speed) = self.speed { speed } else { 13.9 }; // default speed is 13.9 m/s (50 km/h)
        let lanes = if let Some(lanes) = self.num_lanes { lanes as f64 } else { 1.0 };

        // Based on the definitions in PTV-Validate and in the VISUM-Cologne network
        let capacity_per_lane = match road_class {
            0 | 1 => 2000.0, // CR13 in table.py
            2 => {
                if speed <= 11.0 {
                    1333.33 // CR5 in table.py
                } else if speed <= 16.0 {
                    1500.0 // CR3 in table.py
                } else {
                    2000.0 // CR13 in table.py
                }
            }
            3 => {
                if speed <= 11.0 {
                    800.0 // CR5 in table.py
                } else if speed <= 13.0 {
                    875.0 // CR5 in table.py
                } else if speed <= 16.0 {
                    1500.0 // CR4 in table.py
                } else {
                    1800.0 // CR13 in table.py
                }
            }
            _ => {
                // road_class >= 4 or road_class == -1
                if speed <= 5.0 {
                    200.0 // CR7 in table.py
                } else if speed <= 7.0 {
                    412.5 // CR7 in table.py
                } else if speed <= 9.0 {
                    600.0 // CR6 in table.py
                } else if speed <= 11.0 {
                    800.0 // CR5 in table.py
                } else if speed <= 13.0 {
                    1125.0 // CR5 in table.py
                } else if speed <= 16.0 {
                    1583.0 // CR4 in table.py
                } else if speed <= 18.0 {
                    1100.0 // CR3 in table.py
                } else if speed <= 22.0 {
                    1200.0 // CR3 in table.py
                } else if speed <= 26.0 {
                    1300.0 // CR3 in table.py
                } else {
                    1400.0 // CR3 in table.py
                }
            }
        };

        lanes * capacity_per_lane
    }
}

#[derive(Debug, Deserialize)]
pub struct Lane {
    #[serde(rename = "@index")]
    pub index: u32,

    #[serde(rename = "param", default)]
    pub params: Vec<Param>,
}

#[derive(Default, Debug, Deserialize)]
pub struct Param {
    #[serde(rename = "@key")]
    pub key: String,
    #[serde(rename = "@value")]
    pub value: Option<String>,
}
