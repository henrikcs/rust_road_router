use std::path::Path;

use crate::traffic_model::{TrafficModel, TrafficModelType};

pub struct TrafficModelData {
    /// edge i has traffic model traffic_model[i]
    pub traffic_model: Vec<Box<dyn TrafficModel>>,

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

    /// speed observations for each edge stored consecutively
    /// each edge may have a diffrent number of speed observations,
    /// if edge `i` has `k_i` speed observations, then the edge has `k_i` density observations as well
    pub speed_observations: Vec<f64>,
}

impl TrafficModelDataFlattened {
    pub fn reconstruct(dir: &Path, traffic_model_type: TrafficModelType) -> Self {
        // load from files FILE_EDGE_TRAFFIC_MODEL_PARAMS, FILE_EDGE_DENSITY_OBSERVATIONS, FILE_EDGE_SPEED_OBSERVATIONS
    }

    pub fn deconstruct(&self, dir: &Path) -> Result<(), std::io::Error> {
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
}

impl Into<TrafficModelData> for TrafficModelDataFlattened {
    fn into(self) -> TrafficModelData {
        match self.traffic_model_type {
            TrafficModelType::ModifiedLee => {
                // has 5 parameters per edge
                // params[0] = free_flow_speed
                // params[1] = a
                // params[2] = e
                // params[3] = theta
                // params[4] = jam_density
            }
        }
    }
}

impl From<TrafficModelData> for TrafficModelDataFlattened {
    fn from(value: TrafficModelData) -> Self {
        match value.traffic_model_type {
            TrafficModelType::ModifiedLee => {
                // save 5 params in the following order:
                // params[0] = free_flow_speed
                // params[1] = a
                // params[2] = e
                // params[3] = theta
                // params[4] = jam_density
            }
        }
    }
}
