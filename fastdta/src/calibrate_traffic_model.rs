use std::collections::HashMap;

use conversion::sumo::meandata::MeandataDocumentRoot;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::traffic_model::{TrafficModel, TrafficModelType, modified_lee::ModifiedLee};

pub fn calibrate_traffic_model<'a>(
    meandata: &MeandataDocumentRoot,
    edge_ids: &'a [String],
    free_flow_travel_times: &[f64],
    traffic_model_type: &TrafficModelType,
) -> HashMap<&'a String, Box<dyn TrafficModel>> {
    match traffic_model_type {
        TrafficModelType::ModifiedLee => calibrate_modified_lee(meandata, edge_ids, free_flow_travel_times),
    }
}

fn calibrate_modified_lee<'a>(
    meandata: &MeandataDocumentRoot,
    edge_ids: &'a [String],
    free_flow_travel_times: &[f64],
) -> HashMap<&'a String, Box<dyn TrafficModel>> {
    let calibrated_models: HashMap<&'a String, Box<dyn TrafficModel>> = edge_ids
        .par_iter()
        .enumerate()
        .map(|(edge_index, edge_id)| {
            let mut observed_density = Vec::new();
            let mut observed_speed = Vec::new();
            let mut max_density = 0.0;
            for interval in &meandata.intervals {
                if let Some(edge) = interval.edges.iter().find(|e| &e.id == edge_id) {
                    if let (Some(density), Some(speed)) = (edge.lane_density, edge.speed) {
                        observed_density.push(density);
                        observed_speed.push(speed);
                        if density > max_density {
                            max_density = density;
                        }
                    }
                }
            }
            let mut modified_lee = ModifiedLee::new(free_flow_travel_times[edge_index], max_density);

            if !observed_density.is_empty() {
                modified_lee.calibrate(&observed_speed, &observed_density);
            }
            (edge_id, Box::new(modified_lee) as Box<dyn TrafficModel>)
        })
        .collect();

    calibrated_models
}
