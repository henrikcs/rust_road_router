use rayon::prelude::*;

pub fn get_relative_gap(best_tt: &Vec<f64>, simulated_tt: &Vec<f64>) -> f64 {
    assert_eq!(best_tt.len(), simulated_tt.len());

    best_tt
        .par_iter()
        .enumerate()
        .map(|(i, &tt)| {
            if tt == 0.0 {
                return 0.0;
            }

            // if simulated_tt[i] < tt {
            //     return 0.0;
            // }
            debug_assert!(
                simulated_tt[i] > tt,
                "Simulated travel time for trip {} is less than best travel time: {} < {}",
                i,
                simulated_tt[i],
                tt
            );

            simulated_tt[i] - tt
        })
        .sum::<f64>()
        / best_tt.par_iter().sum::<f64>()
}
