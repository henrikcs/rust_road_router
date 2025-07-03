use serde_derive::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename = "routes")]
pub struct RoutesDocumentRoot {
    #[serde(rename = "vehicle")]
    pub vehicles: Vec<Vehicle>,
}

#[derive(Debug, Serialize)]
pub struct Vehicle {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@depart")]
    pub depart: f32,
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

#[derive(Debug, Serialize)]
pub struct Route {
    #[serde(default, rename = "@edges")]
    pub edges: String,
    #[serde(default, rename = "@cost")]
    pub cost: Option<f32>,
    #[serde(default, rename = "@probability")]
    pub probability: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct RouteDistribution {
    #[serde(default, rename = "@last")]
    pub last: u32,
    #[serde(default, rename = "route")]
    pub routes: Vec<Route>,
}
