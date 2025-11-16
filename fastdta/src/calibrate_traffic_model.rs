use std::collections::HashMap;

use conversion::sumo::meandata::MeandataDocumentRoot;

use crate::traffic_model::{TrafficModel, modified_lee::ModifiedLee};

pub fn calibrate_modified_lee<'a>(
    meandata: &MeandataDocumentRoot,
    edge_ids: &'a [String],
    free_flow_travel_times: &[f64],
) -> HashMap<&'a String, Box<dyn TrafficModel>> {
    let mut calibrated_models = HashMap::new();
    // todo: parallelize
    for (edge_index, edge_id) in edge_ids.iter().enumerate() {
        let mut observed_density = Vec::new();
        let mut observed_speed = Vec::new();
        let mut max_density = 0.0;
        for interval in &meandata.intervals {
            if let Some(edge) = interval.edges.iter().find(|e| &e.id == edge_id) {
                if let (Some(density), Some(speed)) = (edge.lanedensity, edge.speed) {
                    observed_density.push(density);
                    observed_speed.push(speed);
                    if density > max_density {
                        max_density = density;
                    }
                }
            }
        }
        let mut modified_lee = ModifiedLee::new(free_flow_travel_times[edge_index], max_density);
        modified_lee.calibrate(&observed_speed, &observed_density);
        calibrated_models.insert(edge_id, Box::new(modified_lee) as Box<dyn TrafficModel>);
    }

    calibrated_models
}
