pub const LOGIT: &str = "logit";
pub const GAWRON: &str = "gawron";

pub enum ChoiceAlgorithm {
    Gawron { a: f64, beta: f64 },
    Logit { theta: f64 },
}

impl ChoiceAlgorithm {
    pub fn create_gawron(a: f64, beta: f64) -> Self {
        ChoiceAlgorithm::Gawron { a, beta }
    }

    pub fn create_logit(theta: f64) -> Self {
        ChoiceAlgorithm::Logit { theta }
    }
}

pub fn gawron(choice_set: &Vec<Vec<u32>>, weights: &Vec<f64>, a: f64, beta: f64) -> Vec<f64> {
    let mut probabilities = vec![1.0; choice_set.len()];

    // TODO: Implement the Gawron choice model

    // for this, we need the previous iteration's travel times, previous probability of chosen route
    // and the current iteration's travel times

    probabilities
}

pub fn logit(costs: &Vec<f64>, theta: f64) -> Vec<f64> {
    // TODO: Implement the logit choice model

    let mut probabilities = vec![0.0; costs.len()];

    let mut sum_weights: f64 = 0.0;

    for &weight in costs {
        sum_weights += f64::from((theta * weight).exp());
    }

    for (i, &cost) in costs.iter().enumerate() {
        probabilities[i] = f64::from((theta * cost).exp()) / sum_weights;
    }

    probabilities
}
