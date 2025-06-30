pub mod meandata;
pub mod network;
pub mod network_reader;
pub mod routes;
pub mod routes_reader;
pub mod routes_writer;

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
