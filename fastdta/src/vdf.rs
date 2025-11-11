pub enum VDFType {
    Ptv,

    Bpr,
}

/// Trait for Volume Delay Functions (VDF)
/// A VDF models the relationship between traffic flow and travel time on a road segment.
pub trait VDF {
    fn travel_time(&self, flow: f64, capacity: f64, free_flow_time: f64) -> f64;

    fn travel_time_estimation(
        &self,
        previous_flow: f64,
        previous_density: f64,
        previous_tt: f64,
        flow: f64,
        density: f64,
        capacity: f64,
        length: f64,
        free_flow_time: f64,
    ) -> f64 {
        if capacity == 0.0 {
            return f64::INFINITY;
        }

        // let estimated_tt = self.travel_time(flow, capacity, free_flow_time);

        let velocity = (flow / density) / 3.6; // in m/s

        let estimated_tt = length / velocity; // in seconds

        // println!(
        //     "Estimating travel time: flow = {}, capacity = {}, free_flow_time = {}, tt = {}",
        //     flow, capacity, free_flow_time, estimated_tt
        // );

        // println!(
        //     "Previous travel time: flow = {}, capacity = {}, free_flow_time = {}, tt = {}",
        //     previous_flow, capacity, free_flow_time, previous_tt
        // );

        estimated_tt
    }
}
/// BPR (Bureau of Public Roads) VDF implementation
///
/// Default Values for alpha and beta:
/// - alpha = 0.15
/// - beta = 4.0
#[derive(Debug, Clone, Copy)]
pub struct Bpr {
    pub alpha: f64,
    pub beta: f64,
}

impl Bpr {
    pub fn create(alpha: f64, beta: f64) -> Self {
        Self { alpha, beta }
    }

    pub fn default() -> Self {
        Self { alpha: 0.15, beta: 4.0 }
    }
}

impl VDF for Bpr {
    fn travel_time(&self, flow: f64, capacity: f64, free_flow_time: f64) -> f64 {
        if capacity == 0.0 {
            return f64::INFINITY;
        }
        free_flow_time * (1.0 + self.alpha * (flow as f64 / capacity as f64).powf(self.beta))
    }
}

// based on the definitions in PTV-Validate and in the VISUM-Cologne network
pub struct Ptv {
    edge_priority: i32,
    edge_speedlimit: f64,
}

impl Ptv {
    pub fn create(edge_priority: i32, edge_speedlimit: f64) -> Self {
        Self {
            edge_priority,
            edge_speedlimit,
        }
    }
}

impl VDF for Ptv {
    fn travel_time(&self, flow: f64, capacity: f64, free_flow_time: f64) -> f64 {
        let road_class = -self.edge_priority;
        let speed = self.edge_speedlimit;

        // Calculate the delay factor based on road class and speed
        // Formula: free_flow_time * (1 + alpha * (flow / (capacity * beta)) * gamma)
        let (alpha, beta, gamma) = match road_class {
            0 | 1 => (1.0, 1.3, 2.0), // CR13 in table.py
            2 => {
                if speed <= 11.0 {
                    (1.0, 0.9, 3.0) // CR5 in table.py
                } else if speed <= 16.0 {
                    (1.0, 1.0, 2.0) // CR3 in table.py
                } else {
                    (1.0, 1.3, 2.0) // CR13 in table.py
                }
            }
            3 => {
                if speed <= 11.0 {
                    (1.0, 0.9, 3.0) // CR5 in table.py
                } else if speed <= 13.0 {
                    (1.0, 0.9, 3.0) // CR5 in table.py
                } else if speed <= 16.0 {
                    (1.7, 1.0, 2.0) // CR4 in table.py
                } else {
                    (1.0, 1.3, 2.0) // CR13 in table.py
                }
            }
            _ => {
                // road_class >= 4 or road_class == -1
                if speed <= 5.0 {
                    (1.0, 0.5, 3.0) // CR7 in table.py
                } else if speed <= 7.0 {
                    (1.0, 0.5, 3.0) // CR7 in table.py
                } else if speed <= 9.0 {
                    (1.0, 0.8, 3.0) // CR6 in table.py
                } else if speed <= 11.0 {
                    (1.0, 0.9, 3.0) // CR5 in table.py
                } else if speed <= 13.0 {
                    (1.0, 0.9, 3.0) // CR5 in table.py
                } else if speed <= 16.0 {
                    (1.7, 1.0, 2.0) // CR4 in table.py
                } else if speed <= 18.0 {
                    (1.0, 1.0, 2.0) // CR3 in table.py
                } else if speed <= 22.0 {
                    (1.0, 1.0, 2.0) // CR3 in table.py
                } else if speed <= 26.0 {
                    (1.0, 1.0, 2.0) // CR3 in table.py
                } else {
                    (1.0, 1.0, 2.0) // CR3 in table.py
                }
            }
        };

        free_flow_time * (1.0 + alpha * (flow / (capacity * beta)) * gamma)
    }
}
