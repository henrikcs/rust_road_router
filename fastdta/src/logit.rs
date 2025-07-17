use crate::alternative_paths::AlternativePaths;

pub fn logit(alternatives: &AlternativePaths, beta: f64, gamma: f64, theta: f64) -> Vec<f64> {
    let n_alternatives = alternatives.paths.len();
    if n_alternatives == 0 {
        return vec![];
    }

    if n_alternatives == 1 {
        return vec![1.0];
    }

    // Calculate theta (if negative, auto-calculate)
    let theta = if theta >= 0.0 { theta } else { get_theta_for_c_logit(&alternatives.costs) };

    // Calculate beta (if negative, auto-calculate)
    let beta = if beta >= 0.0 { beta } else { get_beta_for_c_logit(&alternatives.costs) };

    // Calculate commonalities for c-logit (following C++ implementation)
    let mut commonalities = vec![0.0; n_alternatives];
    if beta > 0.0 {
        for i in 0..n_alternatives {
            let edges_i = &alternatives.paths[i].edges;
            let length_i = alternatives.costs[i]; // Using cost as travel time

            let mut overlap_sum = 0.0;
            for j in 0..n_alternatives {
                let edges_j = &alternatives.paths[j].edges;
                let length_j = alternatives.costs[j];

                // Calculate overlap length between routes i and j
                let overlap_length = calculate_overlap_length(edges_i, edges_j, length_i, length_j);

                // Following C++ formula: pow(overlapLength / sqrt(lengthR * lengthS), gamma)
                overlap_sum += (overlap_length / (length_i * length_j).sqrt()).powf(gamma);
            }
            // Following C++: myCommonalities[pR] = beta * log(overlapSum)
            commonalities[i] = beta * overlap_sum.ln();
        }
    }

    // Calculate probabilities following C++ implementation
    let mut probabilities = vec![0.0; n_alternatives];
    for i in 0..n_alternatives {
        let mut weighted_sum = 0.0;
        for j in 0..n_alternatives {
            // Following C++: theta * (pR->getCosts() - pS->getCosts() + myCommonalities[pR] - myCommonalities[pS])
            let cost_diff = alternatives.costs[i] - alternatives.costs[j];
            let commonality_diff = commonalities[i] - commonalities[j];
            weighted_sum += (theta * (cost_diff + commonality_diff)).exp();
        }
        // Following C++: pR->setProbability(1. / weightedSum)
        probabilities[i] = 1.0 / weighted_sum;
    }

    probabilities
}

fn get_beta_for_c_logit(costs: &[f64]) -> f64 {
    costs.iter().fold(f64::INFINITY, |acc, &cost| acc.min(cost / 3600.0))
}

fn get_theta_for_c_logit(costs: &[f64]) -> f64 {
    if costs.is_empty() {
        return 1.0 / 3600.0;
    }

    // Convert to hours for calculation
    let costs_hours: Vec<f64> = costs.iter().map(|&c| c / 3600.0).collect();

    let sum: f64 = costs_hours.iter().sum();
    let min_cost = costs_hours.iter().fold(f64::INFINITY, |acc, &cost| acc.min(cost));
    let mean_cost = sum / costs_hours.len() as f64;

    let variance: f64 = costs_hours.iter().map(|&cost| (cost - mean_cost).powi(2)).sum::<f64>() / costs_hours.len() as f64;

    let cv_cost = variance.sqrt() / mean_cost;

    // Magic numbers from Lohse book
    if cv_cost > 0.0 {
        std::f64::consts::PI / ((6.0_f64).sqrt() * cv_cost * (min_cost + 1.1)) / 3600.0
    } else {
        1.0 / 3600.0
    }
}

fn calculate_overlap_length(edges_i: &[u32], edges_j: &[u32], length_i: f64, length_j: f64) -> f64 {
    // Following C++ implementation: calculate overlap by finding common edges
    // In C++, this calculates the sum of travel times for common edges

    // If routes are identical, overlap is the minimum of the two lengths
    if edges_i == edges_j {
        return length_i.min(length_j);
    }

    // Count common edges (in C++ this would sum their travel times)
    let mut common_edges = 0;
    for edge_i in edges_i {
        if edges_j.contains(edge_i) {
            common_edges += 1;
        }
    }

    if common_edges == 0 {
        return 0.0;
    }

    // Since we don't have individual edge travel times, we approximate
    // the overlap length by assuming uniform distribution of travel time across edges
    let avg_edge_time_i = length_i / edges_i.len() as f64;
    let avg_edge_time_j = length_j / edges_j.len() as f64;

    // Use the average of the two edge times for common edges
    let avg_common_edge_time = (avg_edge_time_i + avg_edge_time_j) / 2.0;

    // Return the total overlap time
    common_edges as f64 * avg_common_edge_time
}
