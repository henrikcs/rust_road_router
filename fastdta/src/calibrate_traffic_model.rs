use std::collections::HashMap;

use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::traffic_model::{TrafficModel, TrafficModelType, modified_lee::ModifiedLee};

pub fn calibrate_traffic_model<'a>(
    observed_densities_of_edges: &Vec<Vec<f64>>,
    observed_speeds_of_edges: &Vec<Vec<f64>>,
    free_flow_travel_times: &[f64],
    traffic_model_type: &TrafficModelType,
) -> HashMap<usize, Box<dyn TrafficModel>> {
    assert!(observed_densities_of_edges.len() == observed_speeds_of_edges.len());
    assert!(observed_densities_of_edges.len() == free_flow_travel_times.len());

    match traffic_model_type {
        TrafficModelType::ModifiedLee => calibrate_modified_lee(observed_densities_of_edges, observed_speeds_of_edges, free_flow_travel_times),
    }
}

fn calibrate_modified_lee<'a>(
    observed_densities_of_edges: &Vec<Vec<f64>>,
    observed_speeds_of_edges: &Vec<Vec<f64>>,
    free_flow_travel_times: &[f64],
) -> HashMap<usize, Box<dyn TrafficModel>> {
    let calibrated_models: HashMap<usize, Box<dyn TrafficModel>> = free_flow_travel_times
        .par_iter()
        .enumerate()
        .map(|(edge_index, fftt)| {
            let observed_density = &observed_densities_of_edges[edge_index];
            let observed_speed = &observed_speeds_of_edges[edge_index];
            let max_density = if let Some(d) = observed_density.iter().max_by(|a, b| a.total_cmp(b)) {
                *d
            } else {
                0.0
            };

            let mut modified_lee = ModifiedLee::new(*fftt, max_density);

            if !observed_density.is_empty() {
                modified_lee.calibrate(&observed_speed, &observed_density);
            }
            (edge_index, Box::new(modified_lee) as Box<dyn TrafficModel>)
        })
        .collect();

    calibrated_models
}
