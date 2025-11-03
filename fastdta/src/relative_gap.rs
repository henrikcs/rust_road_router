use rayon::prelude::*;

/// two travel times are considered equal if they differ at most by this number
/// Time is given in seconds
pub const EPSILON_TRAVEL_TIME: f64 = 0.001;

pub fn get_relative_gap(best_tts: &Vec<f64>, simulated_tts: &Vec<f64>) -> f64 {
    assert_eq!(best_tts.len(), simulated_tts.len());

    best_tts
        .par_iter()
        .enumerate()
        .map(|(i, &best_tt)| {
            if simulated_tts[i] - best_tt < -EPSILON_TRAVEL_TIME {
                eprintln!(
                    "Simulated travel time for trip {} is less than best travel time: {} < {}",
                    i, simulated_tts[i], best_tt
                );
                return 0.0;
            }

            if f64::abs(simulated_tts[i] - best_tt) < EPSILON_TRAVEL_TIME {
                return 0.0;
            }

            simulated_tts[i] - best_tt
        })
        .sum::<f64>()
        / best_tts.par_iter().sum::<f64>()
}
