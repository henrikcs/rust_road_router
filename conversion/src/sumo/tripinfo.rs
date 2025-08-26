use serde_derive::{Deserialize, Serialize};

use crate::sumo::{SumoTimestamp, SumoTravelTime};

/// <tripinfos xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:noNamespaceSchemaLocation="http://sumo.dlr.de/xsd/tripinfo_file.xsd">
///    <tripinfo id="7" depart="6.00" departLane="G4F4_0" departPos="0.00" departSpeed="12.31" departDelay="0.40" arrival="69.00" arrivalLane="F4F3_0" arrivalPos="385.60" arrivalSpeed="12.31" duration="63.00" routeLength="771.20" waitingTime="0.00" waitingCount="0" stopTime="0.00" timeLoss="0.00" rerouteNo="0" devices="tripinfo_7" vType="DEFAULT_VEHTYPE" speedFactor="0.89" vaporized=""/>
/// </tripinfos>
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "tripinfos")]
pub struct TripinfosDocumentRoot {
    #[serde(rename = "tripinfo", default)]
    pub tripinfos: Vec<Tripinfo>,
}

/// <tripinfo id="7" depart="6.00" departLane="G4F4_0" departPos="0.00" departSpeed="12.31" departDelay="0.40" arrival="69.00" arrivalLane="F4F3_0" arrivalPos="385.60" arrivalSpeed="12.31" duration="63.00" routeLength="771.20" waitingTime="0.00" waitingCount="0" stopTime="0.00" timeLoss="0.00" rerouteNo="0" devices="tripinfo_7" vType="DEFAULT_VEHTYPE" speedFactor="0.89" vaporized=""/>
#[derive(Debug, Deserialize, Serialize)]
pub struct Tripinfo {
    #[serde(rename = "@id")]
    pub id: String,

    #[serde(rename = "@depart")]
    pub depart: SumoTimestamp,

    #[serde(rename = "@duration")]
    pub duration: SumoTravelTime,
}
