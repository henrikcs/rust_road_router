use serde_derive::Serialize;
#[derive(Debug, Serialize)]
#[serde(rename = "routes")]
pub struct Routes {
    #[serde(rename = "@xmlns:xsi")]
    pub xmlns_xsi: String,

    #[serde(rename = "@xsi:noNamespaceSchemaLocation")]
    pub schema_location: String,

    #[serde(rename = "vehicle")]
    pub vehicles: Vec<Vehicle>,
}

#[derive(Debug, Serialize)]
pub struct Vehicle {
    #[serde(rename = "@id")]
    pub id: String,

    #[serde(rename = "@depart")]
    pub depart: String,

    #[serde(rename = "@departLane")]
    pub depart_lane: String,

    #[serde(rename = "@departPos")]
    pub depart_pos: String,

    #[serde(rename = "@departSpeed")]
    pub depart_speed: String,

    #[serde(rename = "route")]
    pub route: Route,
}

#[derive(Debug, Serialize)]
pub struct Route {
    #[serde(rename = "@edges")]
    pub edges: String,
}
