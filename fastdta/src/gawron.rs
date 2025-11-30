use crate::alternative_paths::AlternativePaths;

pub fn gawron(alternatives: &mut AlternativePaths, a: f64, beta: f64, previous_costs: &Vec<f64>) -> Vec<f64> {
    let mut probabilities = alternatives.probabilities.clone();

    for i in 0..alternatives.paths.len() {
        if alternatives.costs[i] < 0.0 {
            probabilities[i] = 0.0;
        }
    }

    if alternatives.paths.len() <= 1 {
        return probabilities;
    }

    // update the cost according to which choice was made
    // prevent remembering the latest travel time of other paths too strongly
    for i in 0..alternatives.paths.len() {
        // previous costs has length at least alternatives.paths.len() - 1
        // if a new path is added, the previous cost is not known, so we skip smoothing in that case
        if i != alternatives.choice && i < previous_costs.len() {
            // smooth the cost for non-chosen alternatives
            alternatives.costs[i] = alternatives.costs[i] * beta + previous_costs[i] * (1.0 - beta);
        }
    }

    // Iterate through all pairs of alternatives (i, j) where i < j
    for i in 0..alternatives.paths.len() {
        for j in (i + 1)..alternatives.paths.len() {
            let cost_i = alternatives.costs[i];
            let cost_j = alternatives.costs[j];
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
        let mut alternatives = AlternativePaths {
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
        let new_probabilities = gawron(&mut alternatives, a, beta, &vec![569.10, 560.00, 555.70, 555.68, 555.90]);
        let expected_probabilities = vec![0.12916244, 0.19314789, 0.22728348, 0.22647978, 0.22392642];
        for (new, expected) in new_probabilities.iter().zip(expected_probabilities.iter()) {
            let diff = (new - expected).abs();
            assert!(diff < 0.0001, "Expected {}, got {}, diff {}", expected, new, diff);
        }
    }

    /*

    previous alternative (in iteration 0):
    <vehicle id="4730" depart="2490.000" departLane="best" departPos="base" departSpeed="max" arrivalPos="max">
        <routeDistribution last="0">
            <route cost="161.664608" traveltime="161.664608" probability="1.00000000" edges="s a1 a2 t" exitTimes="2525.478762 2560.343413 2610.287257 2646.164867"/>
        </routeDistribution>
    </vehicle>

    expected alternatives (in iteration 1):
    <vehicle id="4730" depart="2490.000" departLane="best" departPos="base" departSpeed="max" arrivalPos="max">
        <routeDistribution last="0">
            <route cost="347.208207" traveltime="347.208207" probability="0.49007452" edges="s a1 a2 t" exitTimes="2657.878726 2731.309021 2791.958606 2831.708466"/>
            <route cost="296.413777" traveltime="296.413777" probability="0.50992548" edges="s b1 b2 t" exitTimes="2657.878726 2693.007596 2742.834090 2780.852191"/>
        </routeDistribution>
    </vehicle>

    expected alternatives (in iteration 2):
    <vehicle id="4730" depart="2490.000" departLane="best" departPos="base" departSpeed="max" arrivalPos="max">
        <routeDistribution last="0">
            <route cost="178.604462" traveltime="178.604462" probability="0.52078601" edges="s a1 a2 t" exitTimes="2527.681896 2567.783924 2622.233994 2663.104721"/>
            <route cost="286.820482" traveltime="285.7545611" probability="0.47921399" edges="s b1 b2 t" exitTimes="2527.681896 2659.856424 2726.727551 2770.192974"/>
        </routeDistribution>
    </vehicle>

    with beta = 0.9, a = 0.5
     */

    #[test]
    fn test_gawron_multiple_iterations() {
        let beta = 0.9;
        let a = 0.5;

        // from iteration 0 (given) i want to reach the same values as in iteration 1
        let mut alternatives_iter1 = AlternativePaths {
            paths: vec![AlternativePath { edges: vec![] }, AlternativePath { edges: vec![] }],
            choice: 0,
            costs: vec![347.208207, 296.413777],
            probabilities: vec![0.5, 0.5],
        };

        let previous_costs_iter1 = vec![161.664608];

        let new_probabilities_iter1 = gawron(&mut alternatives_iter1, a, beta, &previous_costs_iter1);
        let expected_probabilities_iter1 = vec![0.49007452, 0.50992548];
        for (new, expected) in new_probabilities_iter1.iter().zip(expected_probabilities_iter1.iter()) {
            let diff = (new - expected).abs();
            assert!(diff < 0.0001, "Iteration 1: Expected {}, got {}, diff {}", expected, new, diff);
        }

        // from iteration 1 to iteration 2
        let mut alternatives_iter2 = AlternativePaths {
            paths: vec![AlternativePath { edges: vec![] }, AlternativePath { edges: vec![] }],
            choice: 0,
            costs: vec![178.604462, 285.7545611],
            probabilities: new_probabilities_iter1,
        };

        let previous_costs_iter2 = vec![347.208207, 296.413777];

        let new_probabilities_iter2 = gawron(&mut alternatives_iter2, a, beta, &previous_costs_iter2);
        let expected_probabilities_iter2 = vec![0.52078601, 0.47921399];
        for (new, expected) in new_probabilities_iter2.iter().zip(expected_probabilities_iter2.iter()) {
            let diff = (new - expected).abs();
            assert!(diff < 0.0001, "Iteration 2: Expected {}, got {}, diff {}", expected, new, diff);
        }
    }

    #[test]
    fn test_grawron_higher_costs_less_probability() {
        let mut alternatives = AlternativePaths {
            paths: vec![AlternativePath { edges: vec![] }, AlternativePath { edges: vec![] }],
            choice: 0,
            costs: vec![350.0, 300.0],
            // Initial probabilities should be properly scaled when a new route is added
            probabilities: vec![0.5, 0.5],
        };

        let a = 0.5;
        let beta = 0.9;
        let new_probabilities = gawron(&mut alternatives, a, beta, &vec![350.0]);

        assert!(alternatives.costs[0] == 350.0, "Expected cost not change, got {}", alternatives.costs[0]);
        assert!(alternatives.costs[1] == 300.0, "Expected cost not change, got {}", alternatives.costs[1]);

        // route with higher cost should have lower probability
        assert!(
            new_probabilities[0] < new_probabilities[1],
            "Expected first probability to be lower than second, got {} and {}",
            new_probabilities[0],
            new_probabilities[1]
        );
    }
}
