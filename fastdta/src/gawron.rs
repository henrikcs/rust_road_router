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
