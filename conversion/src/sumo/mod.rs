use std::{error::Error, path::Path};

use rust_road_router::datastr::graph::{floating_time_dependent::IPPIndex, EdgeId, NodeId};

use crate::{SerializedTimestamp, SerializedTravelTime};

pub mod base_types;
pub mod connections;
pub mod connections_writer;
pub mod edges;
pub mod edges_reader;
pub mod meandata;
pub mod meandata_reader;
pub mod nodes;
pub mod nodes_reader;
pub mod paths_to_sumo_routes_converter;
pub mod routes;
pub mod routes_reader;
pub mod routes_writer;
pub mod sumo_to_new_graph_weights;
pub mod sumo_to_td_graph_converter;
pub mod trips;
pub mod trips_reader;

pub const EDG_XML: &str = ".edg.xml";
pub const NOD_XML: &str = ".nod.xml";
pub const CON_XML: &str = ".con.xml";
pub const TRIPS_XML: &str = ".trips.xml";
pub const ROUTES: &str = ".rou.xml";
pub const ALT_ROUTES: &str = ".rou.alt.xml";

/// in seconds
pub type SumoTravelTime = f64;
/// in seconds
pub type SumoTimestamp = f64;
/// in meters
pub type SumoPosition = f64;

/// Implicit time dependent graph representation defined by RoutingKit.
/// This is a tuple of:
/// - `first_out`: first out index for each node (length n+1)
/// - `head`: head node id for each edge (length m)
/// - `first_ipp_of_arc`: first interpolation point index for each edge (length m+1)
/// - `ipp_departure_time`: departure time for each interpolation point in milliseconds (length m)
/// - `ipp_travel_time`: travel time for each interpolation point in milliseconds (length m)
pub type RoutingKitTDGraph = (Vec<EdgeId>, Vec<NodeId>, Vec<IPPIndex>, Vec<SerializedTimestamp>, Vec<SerializedTravelTime>);

pub trait XmlReader {
    type R;

    fn read(file: &Path) -> Result<Self::R, Box<dyn Error>>;
}

pub trait XmlWriter {
    type R;

    fn write(output_file: &Path, doc: &Self::R) -> Result<(), Box<dyn Error>>;
}
