use serde_derive::Deserialize;

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
    pub begin: f32,
    #[serde(rename = "@end")]
    pub end: f32,

    #[serde(rename = "edge")]
    pub edges: Vec<Edge>,
}

#[derive(Debug, Deserialize)]
pub struct Edge {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@traveltime")]
    pub traveltime: f32,
}
