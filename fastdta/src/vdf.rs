/// Trait for Volume Delay Functions (VDF)
/// A VDF models the relationship between traffic flow and travel time on a road segment.
pub trait VDF {
    fn travel_time(&self, flow: u32, capacity: u32, free_flow_time: f64) -> f64;
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
    pub fn new(alpha: f64, beta: f64) -> Self {
        Self { alpha, beta }
    }

    pub fn default() -> Self {
        Self { alpha: 0.15, beta: 4.0 }
    }
}

impl VDF for Bpr {
    fn travel_time(&self, flow: u32, capacity: u32, free_flow_time: f64) -> f64 {
        if capacity == 0 {
            return f64::INFINITY;
        }
        free_flow_time * (1.0 + self.alpha * (flow as f64 / capacity as f64).powf(self.beta))
    }
}
