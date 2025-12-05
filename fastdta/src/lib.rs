use rand::{Rng, SeedableRng, rngs};

pub mod alternative_paths;
pub mod calibrate_traffic_model;
pub mod choice;
pub mod cli;
pub mod customize;
pub mod gawron;
pub mod logger;
pub mod logit;
pub mod path_processor;
pub mod postprocess;
pub mod preprocess;
pub mod preprocess_routes;
pub mod query;
pub mod relative_gap;
pub mod sampled_queries;
pub mod sampled_queries_sumo;
pub mod sampler;
pub mod sumo_runner;
pub mod traffic_model;
pub mod traffic_model_data;

pub fn calculate_keep_routes(n: usize, keep_route_probability: f64, seed: i32) -> Vec<bool> {
    if keep_route_probability <= 0.0 {
        vec![false; n]
    } else if keep_route_probability >= 1.0 {
        vec![true; n]
    } else {
        let mut rng: rngs::StdRng = SeedableRng::seed_from_u64(seed.abs() as u64);
        (0..n).map(|_| rng.random_bool(keep_route_probability)).collect()
    }
}
