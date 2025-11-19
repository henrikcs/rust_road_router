use conversion::sumo::meandata::MeandataDocumentRoot;

use crate::traffic_model_data::TrafficModelData;

pub fn calibrate_traffic_models(traffic_model_data: &mut TrafficModelData, meandata: &mut MeandataDocumentRoot, edge_ids: &Vec<String>, threshold: usize) {
    // for each edge, find data in meandata and add to traffic_model_data

    for (edge_index, edge_id) in edge_ids.iter().enumerate() {
        if traffic_model_data.observed_densities[edge_index].len() < threshold {
            let mut densities: Vec<f64> = Vec::new();
            let mut speeds: Vec<f64> = Vec::new();

            for interval in meandata.intervals.iter_mut() {
                if let Some(edge) = interval.get_edge(edge_id) {
                    if edge.lane_density.is_none() || edge.speed.is_none() {
                        continue;
                    }

                    densities.push(edge.lane_density.unwrap());
                    speeds.push(edge.speed.unwrap() * 3.6); // convert m/s to km/h
                }
            }

            if edge_id == "a2" {
                println!(
                    "Currently {} data points for edge {}",
                    traffic_model_data.observed_densities[edge_index].len(),
                    edge_id
                );
                println!("Densities: {:?}", &traffic_model_data.observed_densities[edge_index]);
                println!("Speeds: {:?}", &traffic_model_data.observed_speeds[edge_index]);
                traffic_model_data.traffic_models[edge_index].debug()
            }

            traffic_model_data.observed_densities[edge_index].extend_from_slice(&densities);
            traffic_model_data.observed_speeds[edge_index].extend_from_slice(&speeds);

            traffic_model_data.calibrate_model(edge_index);

            if edge_id == "a2" {
                println!("Added {} data points for edge {}", densities.len(), edge_id);
                println!("Densities: {:?}", &densities);
                println!("Speeds: {:?}", &speeds);
                traffic_model_data.traffic_models[edge_index].debug()
            }
        }
    }
}
