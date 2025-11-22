use conversion::sumo::meandata::MeandataDocumentRoot;
use std::fs::OpenOptions;
use std::io::Write;

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

            if traffic_model_data.observed_densities[edge_index].is_empty() {
                continue;
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

            traffic_model_data.calibrate_model(edge_index);

            if edge_id == "a2" {
                println!("Added {} data points for edge {}", densities.len(), edge_id);
                println!("Densities: {:?}", &densities);
                println!("Speeds: {:?}", &speeds);
                traffic_model_data.traffic_models[edge_index].debug();

                // Write traffic model data to file for analysis
                if let Err(e) = write_traffic_model_data(edge_id, traffic_model_data, edge_index) {
                    eprintln!("Failed to write traffic model data for edge {}: {}", edge_id, e);
                }
            }
        }
    }
}

fn write_traffic_model_data(edge_id: &str, traffic_model_data: &TrafficModelData, edge_index: usize) -> std::io::Result<()> {
    let filename = format!("traffic_model_data_{}", edge_id);
    let mut file = OpenOptions::new().create(true).append(true).open(&filename)?;

    // Write traffic model type
    let model_type = match traffic_model_data.traffic_model_type {
        crate::traffic_model::TrafficModelType::ModifiedLee => "ModifiedLee",
    };

    // Get parameters
    let params = traffic_model_data.traffic_models[edge_index].get_params_as_vec();

    // Write header format: INVOCATION|<model_type>
    writeln!(file, "INVOCATION|{}", model_type)?;

    // Write parameters (for ModifiedLee: free_flow_speed, a, e, theta, jam_density)
    writeln!(file, "PARAMS|{}", params.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(","))?;

    // Write observed densities
    let densities = &traffic_model_data.observed_densities[edge_index];
    writeln!(file, "DENSITIES|{}", densities.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(","))?;

    // Write observed speeds
    let speeds = &traffic_model_data.observed_speeds[edge_index];
    writeln!(file, "SPEEDS|{}", speeds.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(","))?;

    writeln!(file, "---")?;

    Ok(())
}
