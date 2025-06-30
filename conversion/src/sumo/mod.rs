pub mod network;
pub mod network_reader;
pub mod routes;
pub mod routes_writer;
pub mod trips;
pub mod trips_reader;

#[derive(Default, Debug)]
pub struct Vehicle {
    pub id: String,
    pub departure: u32,
    pub route: Route,
}

#[derive(Default, Debug)]
pub struct Route {
    pub paths: Vec<String>,
}
