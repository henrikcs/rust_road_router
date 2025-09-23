use crate::alternative_paths::AlternativePaths;

pub fn gawron(alternatives: &AlternativePaths, a: f64, _beta: f64) -> Vec<f64> {
    let mut probabilities = alternatives.probabilities.clone();
    let costs = &alternatives.costs;

    if alternatives.paths.len() <= 1 {
        return probabilities;
    }

    // Iterate through all pairs of alternatives (i, j) where i < j
    for i in 0..alternatives.paths.len() {
        for j in (i + 1)..alternatives.paths.len() {
            let cost_i = costs[i];
            let cost_j = costs[j];
            let prob_i = probabilities[i];
            let prob_j = probabilities[j];

            // Calculate delta as in Gawron (1998) equation (4.2)
            let delta = (cost_j - cost_i) / (cost_j + cost_i);

            // Calculate new probabilities using Gawron equations (4.3a, 4.3b)
            let new_prob_i = gawron_f(prob_i, prob_j, delta, a);
            let new_prob_j = prob_i + prob_j - new_prob_i;

            // Handle NaN cases and clamp probabilities to [0, 1]
            let (final_prob_i, final_prob_j) = if new_prob_i.is_nan() || new_prob_j.is_nan() {
                if cost_j > cost_i { (1.0, 0.0) } else { (0.0, 1.0) }
            } else {
                (new_prob_i.clamp(0.0, 1.0), new_prob_j.clamp(0.0, 1.0))
            };

            probabilities[i] = final_prob_i;
            probabilities[j] = final_prob_j;
        }
    }

    probabilities
}

/// Performs the Gawron f() function from "Dynamic User Equilibria..."
fn gawron_f(pdr: f64, pds: f64, x: f64, a: f64) -> f64 {
    let g_val = gawron_g(a, x);
    let denominator = pdr * g_val + pds;

    if denominator == 0.0 {
        return f64::MAX;
    }

    (pdr * (pdr + pds) * g_val) / denominator
}

/// Performs the Gawron g() function from "Dynamic User Equilibria..."
fn gawron_g(a: f64, x: f64) -> f64 {
    let denominator = 1.0 - (x * x);

    if denominator == 0.0 {
        return f64::MAX;
    }

    ((a * x) / denominator).exp()
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use crate::alternative_paths::{AlternativePath, AlternativePaths};

    // previous alternative paths:
    // cost="569.10" probability="0.13094327"
    // cost="560.00" probability="0.19342774"
    // cost="555.70" probability="0.22657958"
    // cost="555.68" probability="0.22577343"
    // cost="555.90" probability="0.22327598"
    // chosen: index 4
    // a = 0.5 (beta = 0.9)

    // expected:
    // cost="569.10" probability="0.12916244"
    // cost="560.00" probability="0.19314789"
    // cost="555.70" probability="0.22728348"
    // cost="555.68" probability="0.22647978"
    // cost="555.90" probability="0.22392642"

    #[test]
    fn test_gawron() {
        let alternatives = AlternativePaths {
            paths: vec![
                AlternativePath { edges: vec![] },
                AlternativePath { edges: vec![] },
                AlternativePath { edges: vec![] },
                AlternativePath { edges: vec![] },
                AlternativePath { edges: vec![] },
            ],
            choice: 4,
            costs: vec![569.10, 560.00, 555.70, 555.68, 555.90],
            probabilities: vec![0.13094327, 0.19342774, 0.22657958, 0.22577343, 0.22327598],
        };

        let a = 0.5;
        let beta = 0.9;
        let new_probabilities = gawron(&alternatives, a, beta);
        let expected_probabilities = vec![0.12916244, 0.19314789, 0.22728348, 0.22647978, 0.22392642];
        for (new, expected) in new_probabilities.iter().zip(expected_probabilities.iter()) {
            let diff = (new - expected).abs();
            assert!(diff < 0.0001, "Expected {}, got {}, diff {}", expected, new, diff);
        }
    }
}
