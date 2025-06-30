use serde_derive::Deserialize;

//TODO: implement http://sumo.dlr.de/xsd/routes_file.xsd

#[derive(Debug, Deserialize)]
#[serde(rename = "routes")]
pub struct Trips {
    #[serde(rename = "trip")]
    pub trips: Vec<Trip>,
}

#[derive(Debug, Deserialize)]
pub struct Trip {
    #[serde(rename = "@id")]
    pub id: String,

    /// the departure time as a floating point number with two decimals
    #[serde(rename = "@depart")]
    pub depart: String,

    /// which lane should be used to start the trip (usually "best")
    #[serde(rename = "@departLane")]
    pub depart_lane: String,

    /// position on the link between 0 and 1 where 0 is the "from" node
    /// and 1 is the to node
    #[serde(rename = "@departPos")]
    pub depart_pos: String,

    /// which speed the vehicle should ride (usually "max")
    #[serde(rename = "@departSpeed")]
    pub depart_speed: String,

    /// Edge (sic!) id where the vehicle starts from
    #[serde(rename = "@from")]
    pub from: String,

    /// Edge (sic!) id where the vehicle is supposed to move to
    #[serde(rename = "@to")]
    pub to: String,
}
