pub const LOGIT: &str = "logit";
pub const GAWRON: &str = "gawron";

pub enum ChoiceAlgorithm {
    Gawron { a: f64, beta: f64 },
    Logit { beta: f64, gamma: f64, theta: f64 },
}

impl ChoiceAlgorithm {
    pub fn create_gawron(a: f64, beta: f64) -> Self {
        ChoiceAlgorithm::Gawron { a, beta }
    }

    pub fn create_logit(beta: f64, gamma: f64, theta: f64) -> Self {
        ChoiceAlgorithm::Logit { beta, gamma, theta }
    }
}
