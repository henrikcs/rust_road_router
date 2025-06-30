use serde_derive::Deserialize;
// TODO: implement http://sumo.dlr.de/xsd/net_file.xsd
#[derive(Deserialize, Debug)]
#[serde(rename = "net")]
pub struct Network {
    pub nodes: Nodes,
    pub links: Links,
}

#[derive(Deserialize, Debug)]
pub struct Nodes {
    #[serde(rename = "node")]
    pub nodes: Vec<Node>,
}

#[derive(Deserialize, Debug)]
pub struct Links {
    #[serde(rename = "link")]
    pub links: Vec<Link>,
}

#[derive(Deserialize, Debug)]
pub struct Node {
    #[serde(rename = "@id")]
    pub id: String,
}

#[derive(Deserialize, Debug)]
pub struct Link {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@from")]
    pub from: String,
    #[serde(rename = "@to")]
    pub to: String,
    #[serde(rename = "@length")]
    pub length: f64,
    #[serde(rename = "@freespeed")]
    pub speed: f64,
}
