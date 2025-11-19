use std::path::Path;

use conversion::{
    FILE_EDGE_DENSITY_OBSERVATIONS, FILE_EDGE_FIRST_DENSITY_OBSERVATION, FILE_EDGE_FIRST_MODEL_PARAM, FILE_EDGE_FIRST_SPEED_OBSERVATION,
    FILE_EDGE_SPEED_OBSERVATIONS, FILE_EDGE_TRAFFIC_MODEL_PARAMS,
};
use rust_road_router::io::{Load, Store};

use crate::traffic_model::{TrafficModel, TrafficModelType, modified_lee::ModifiedLee};

pub struct TrafficModelData {
    /// edge i has traffic model traffic_model[i]
    pub traffic_models: Vec<Box<dyn TrafficModel>>,

    /// observed densities for each edge
    pub observed_densities: Vec<Vec<f64>>,

    /// observed speeds for each edge
    pub observed_speeds: Vec<Vec<f64>>,

    /// traffic model type used for all edges
    pub traffic_model_type: TrafficModelType,
}

/// datastructure to hold traffic model data for each edge used in fastdta
///
/// This structure is used to store the observed densities and speeds for each edge,
/// and the parameters of the traffic model for each edge.
///
#[derive(Debug, Clone)]
struct TrafficModelDataFlattened {
    /// traffic model type used for all edges
    pub traffic_model_type: TrafficModelType,

    /// edge with index `i` has its model parameters stored
    /// starting from index `first_model_param_of_edge[i]` in `model_params`
    /// the number of parameters per edge depends on the traffic model used
    /// for example, ModifiedLee has 5 parameters per edge
    pub first_model_param_of_edge: Vec<usize>,

    /// edge with index `i` has its density observations stored
    /// starting from index `first_density_observation_of_edge[i]` in `density_observations`
    pub first_density_observation_of_edge: Vec<usize>,

    /// edge with index `i` has its speed observations stored
    /// starting from index `first_speed_observation_of_edge[i]` in `speed_observations`
    pub first_speed_observation_of_edge: Vec<usize>,

    /// model parameters for each edge stored consecutively
    /// each edge has `n` parameters, where `n` depends on the traffic model used
    pub model_params: Vec<f64>,

    /// density observations for each edge stored consecutively
    /// each edge may have a diffrent number of density observations,
    /// if edge `i` has `k_i` density observations, then the edge has `k_i` speed observations as well
    pub density_observations: Vec<f64>,

    /// speed observations (in km/h) for each edge stored consecutively
    /// each edge may have a diffrent number of speed observations,
    /// if edge `i` has `k_i` speed observations, then the edge has `k_i` density observations as well
    pub speed_observations: Vec<f64>,
}

impl TrafficModelDataFlattened {
    pub fn reconstruct(dir: &Path, traffic_model_type: TrafficModelType) -> Self {
        // load from files FILE_EDGE_TRAFFIC_MODEL_PARAMS, FILE_EDGE_DENSITY_OBSERVATIONS, FILE_EDGE_SPEED_OBSERVATIONS
        let model_params: Vec<f64> = Vec::<f64>::load_from(dir.join(FILE_EDGE_TRAFFIC_MODEL_PARAMS)).unwrap();
        let density_observations: Vec<f64> = Vec::<f64>::load_from(dir.join(FILE_EDGE_DENSITY_OBSERVATIONS)).unwrap();
        let speed_observations: Vec<f64> = Vec::<f64>::load_from(dir.join(FILE_EDGE_SPEED_OBSERVATIONS)).unwrap();

        // Load the index vectors
        let first_model_param_of_edge: Vec<usize> = Vec::<usize>::load_from(dir.join(FILE_EDGE_FIRST_MODEL_PARAM)).unwrap();
        let first_density_observation_of_edge: Vec<usize> = Vec::<usize>::load_from(dir.join(FILE_EDGE_FIRST_DENSITY_OBSERVATION)).unwrap();
        let first_speed_observation_of_edge: Vec<usize> = Vec::<usize>::load_from(dir.join(FILE_EDGE_FIRST_SPEED_OBSERVATION)).unwrap();

        Self {
            traffic_model_type,
            first_model_param_of_edge,
            first_density_observation_of_edge,
            first_speed_observation_of_edge,
            model_params,
            density_observations,
            speed_observations,
        }
    }

    pub fn deconstruct(&self, dir: &Path) -> Result<(), std::io::Error> {
        self.model_params.write_to(&dir.join(FILE_EDGE_TRAFFIC_MODEL_PARAMS))?;
        self.density_observations.write_to(&dir.join(FILE_EDGE_DENSITY_OBSERVATIONS))?;
        self.speed_observations.write_to(&dir.join(FILE_EDGE_SPEED_OBSERVATIONS))?;

        // Write the index vectors
        self.first_model_param_of_edge.write_to(&dir.join(FILE_EDGE_FIRST_MODEL_PARAM))?;
        self.first_density_observation_of_edge
            .write_to(&dir.join(FILE_EDGE_FIRST_DENSITY_OBSERVATION))?;
        self.first_speed_observation_of_edge.write_to(&dir.join(FILE_EDGE_FIRST_SPEED_OBSERVATION))?;

        Ok(())
    }
}

impl TrafficModelData {
    pub fn reconstruct(dir: &Path, traffic_model_type: TrafficModelType) -> Self {
        TrafficModelDataFlattened::reconstruct(dir, traffic_model_type).into()
    }

    pub fn deconstruct(self, dir: &Path) -> Result<(), std::io::Error> {
        if !dir.exists() {
            std::fs::create_dir(dir)?;
        }
        let flattened: TrafficModelDataFlattened = self.into();
        flattened.deconstruct(dir)
    }

    pub fn calibrate_model(&mut self, index: usize) {
        self.traffic_models[index].calibrate(&self.observed_speeds[index], &self.observed_densities[index]);
    }

    /// initialize traffic model data with empty observations and traffic models with given free flow speeds in km/h
    pub fn init(free_flow_speed: &Vec<f64>, traffic_model_type: TrafficModelType) -> Self {
        let mut traffic_models: Vec<Box<dyn TrafficModel>> = Vec::with_capacity(free_flow_speed.len());
        for ffs in free_flow_speed {
            match traffic_model_type {
                TrafficModelType::ModifiedLee => {
                    traffic_models.push(Box::new(ModifiedLee::new(*ffs, 0.0)));
                }
            }
        }

        TrafficModelData {
            traffic_models: traffic_models,
            observed_densities: vec![vec![]; free_flow_speed.len()],
            observed_speeds: vec![vec![]; free_flow_speed.len()],
            traffic_model_type: traffic_model_type,
        }
    }
}

impl Into<TrafficModelData> for TrafficModelDataFlattened {
    fn into(self) -> TrafficModelData {
        let num_edges = self.first_model_param_of_edge.len() - 1;
        let mut traffic_model: Vec<Box<dyn TrafficModel>> = Vec::new();
        let mut observed_densities: Vec<Vec<f64>> = Vec::new();
        let mut observed_speeds: Vec<Vec<f64>> = Vec::new();

        for i in 0..num_edges {
            // Extract model parameters for edge i
            let start_param = self.first_model_param_of_edge[i];
            let end_param = self.first_model_param_of_edge[i + 1];
            let params: Vec<f64> = self.model_params[start_param..end_param].to_vec();

            match self.traffic_model_type {
                TrafficModelType::ModifiedLee => {
                    // has 5 parameters per edge
                    // params[0] = free_flow_speed
                    // params[1] = a
                    // params[2] = e
                    // params[3] = theta
                    // params[4] = jam_density
                    // Create traffic model from parameters
                    let model = ModifiedLee::from_vec(&params);
                    traffic_model.push(Box::new(model));
                }
            }

            // Extract density observations for edge i
            let start_density = self.first_density_observation_of_edge[i];
            let end_density = self.first_density_observation_of_edge[i + 1];
            observed_densities.push(self.density_observations[start_density..end_density].to_vec());

            // Extract speed observations for edge i
            let start_speed = self.first_speed_observation_of_edge[i];
            let end_speed = self.first_speed_observation_of_edge[i + 1];
            observed_speeds.push(self.speed_observations[start_speed..end_speed].to_vec());
        }

        TrafficModelData {
            traffic_models: traffic_model,
            observed_densities,
            observed_speeds,
            traffic_model_type: self.traffic_model_type,
        }
    }
}

impl From<TrafficModelData> for TrafficModelDataFlattened {
    fn from(value: TrafficModelData) -> Self {
        let mut model_params = Vec::new();
        let mut first_model_param_of_edge = Vec::new();
        let mut density_observations = Vec::new();
        let mut first_density_observation_of_edge = Vec::new();
        let mut speed_observations = Vec::new();
        let mut first_speed_observation_of_edge = Vec::new();

        let mut added_params = 0;
        let mut added_density_observations = 0;
        let mut added_speed_observations = 0;

        for i in 0..value.traffic_models.len() {
            // Store the starting index for model parameters
            first_model_param_of_edge.push(added_params);

            // Extract parameters using the trait method
            let params = value.traffic_models[i].get_params_as_vec();
            model_params.extend(params.clone());
            added_params += params.len();

            // Store density observations
            first_density_observation_of_edge.push(added_density_observations);
            density_observations.extend(&value.observed_densities[i]);
            added_density_observations += value.observed_densities[i].len();

            // Store speed observations
            first_speed_observation_of_edge.push(added_speed_observations);
            speed_observations.extend(&value.observed_speeds[i]);
            added_speed_observations += value.observed_speeds[i].len();
        }

        // Add the final indices
        first_model_param_of_edge.push(added_params);
        first_density_observation_of_edge.push(added_density_observations);
        first_speed_observation_of_edge.push(added_speed_observations);

        Self {
            traffic_model_type: value.traffic_model_type,
            first_model_param_of_edge,
            first_density_observation_of_edge,
            first_speed_observation_of_edge,
            model_params,
            density_observations,
            speed_observations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_traffic_model_data_flattened() {
        // Create a simple traffic model data structure
        let model1 = Box::new(ModifiedLee::from_vec(&vec![13.6, 2.5, 3.1, 2.2, 60.0]));
        let model2 = Box::new(ModifiedLee::from_vec(&vec![15.0, 2.0, 2.8, 1.9, 50.0]));

        let traffic_model_data = TrafficModelData {
            traffic_models: vec![model1, model2],
            observed_densities: vec![vec![10.0, 20.0], vec![15.0]],
            observed_speeds: vec![vec![12.0, 11.0], vec![13.0]],
            traffic_model_type: TrafficModelType::ModifiedLee,
        };

        let flattened: TrafficModelDataFlattened = traffic_model_data.into();

        // Check that the flattened structure is correct
        assert_eq!(flattened.model_params.len(), 10); // 2 edges * 5 params each
        assert_eq!(flattened.first_model_param_of_edge, vec![0, 5, 10]);

        assert_eq!(flattened.density_observations.len(), 3); // 2 + 1
        assert_eq!(flattened.first_density_observation_of_edge, vec![0, 2, 3]);

        assert_eq!(flattened.speed_observations.len(), 3); // 2 + 1
        assert_eq!(flattened.first_speed_observation_of_edge, vec![0, 2, 3]);

        // Check parameter values for first edge
        assert_eq!(flattened.model_params[0], 13.6);
        assert_eq!(flattened.model_params[1], 2.5);
    }

    #[test]
    fn test_from_traffic_model_data_flattened() {
        let flattened = TrafficModelDataFlattened {
            traffic_model_type: TrafficModelType::ModifiedLee,
            first_model_param_of_edge: vec![0, 5, 10],
            first_density_observation_of_edge: vec![0, 2, 3],
            first_speed_observation_of_edge: vec![0, 2, 3],
            model_params: vec![13.6, 2.5, 3.1, 2.2, 60.0, 15.0, 2.0, 2.8, 1.9, 50.0],
            density_observations: vec![10.0, 20.0, 15.0],
            speed_observations: vec![12.0, 11.0, 13.0],
        };

        let traffic_model_data: TrafficModelData = flattened.into();

        // Check that we have 2 edges
        assert_eq!(traffic_model_data.traffic_models.len(), 2);
        assert_eq!(traffic_model_data.observed_densities.len(), 2);
        assert_eq!(traffic_model_data.observed_speeds.len(), 2);

        // Check observations for first edge
        assert_eq!(traffic_model_data.observed_densities[0], vec![10.0, 20.0]);
        assert_eq!(traffic_model_data.observed_speeds[0], vec![12.0, 11.0]);

        // Check observations for second edge
        assert_eq!(traffic_model_data.observed_densities[1], vec![15.0]);
        assert_eq!(traffic_model_data.observed_speeds[1], vec![13.0]);

        // Check that model parameters are correct by converting back to vec
        let params1 = traffic_model_data.traffic_models[0].get_params_as_vec();
        assert_eq!(params1, vec![13.6, 2.5, 3.1, 2.2, 60.0]);

        let params2 = traffic_model_data.traffic_models[1].get_params_as_vec();
        assert_eq!(params2, vec![15.0, 2.0, 2.8, 1.9, 50.0]);
    }

    #[test]
    fn test_roundtrip_conversion() {
        // Create original data
        let model1 = Box::new(ModifiedLee::from_vec(&vec![13.6, 2.5, 3.1, 2.2, 60.0]));
        let model2 = Box::new(ModifiedLee::from_vec(&vec![15.0, 2.0, 2.8, 1.9, 50.0]));

        let original = TrafficModelData {
            traffic_models: vec![model1, model2],
            observed_densities: vec![vec![10.0, 20.0], vec![15.0]],
            observed_speeds: vec![vec![12.0, 11.0], vec![13.0]],
            traffic_model_type: TrafficModelType::ModifiedLee,
        };

        // Convert to flattened and back
        let flattened: TrafficModelDataFlattened = original.into();
        let reconstructed: TrafficModelData = flattened.into();

        // Verify the roundtrip preserved the data
        assert_eq!(reconstructed.traffic_models.len(), 2);
        assert_eq!(reconstructed.observed_densities, vec![vec![10.0, 20.0], vec![15.0]]);
        assert_eq!(reconstructed.observed_speeds, vec![vec![12.0, 11.0], vec![13.0]]);

        let params1 = reconstructed.traffic_models[0].get_params_as_vec();
        assert_eq!(params1, vec![13.6, 2.5, 3.1, 2.2, 60.0]);
    }
}
