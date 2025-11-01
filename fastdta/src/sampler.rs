use rand::{SeedableRng, seq::IndexedRandom};

pub fn sample(sample_relative_sizes: &Vec<f64>, n: usize, seed: i32) -> Vec<Vec<usize>> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed as u64);
    let mut remaining_indices: Vec<usize> = (0..n).collect();
    let mut samples: Vec<Vec<usize>> = Vec::new();
    let total_relative_size: f64 = sample_relative_sizes.iter().sum();

    for (i, &relative_size) in sample_relative_sizes.iter().enumerate() {
        // For the last sample, take all remaining indices to avoid rounding errors
        let sample_size = if i == sample_relative_sizes.len() - 1 {
            remaining_indices.len()
        } else {
            usize::min(((relative_size / total_relative_size) * (n as f64)).round() as usize, remaining_indices.len())
        };

        // let sample: Vec<usize> = if i == 0 {
        //     vec![871]
        // } else {
        let sample: Vec<usize> = remaining_indices.choose_multiple(&mut rng, sample_size).cloned().collect();
        // };

        // Remove sampled indices from remaining_indices

        // if relative_size == 1.0 {
        //     println!("Sample contains {:?} ", sample);
        // }

        samples.push(sample.clone());
        remaining_indices.retain(|index| !sample.contains(index));
    }

    debug_assert_eq!(
        samples.iter().map(|s| s.len()).sum::<usize>(),
        n,
        "Sampled indices do not sum up to total number of indices"
    );

    samples
}

#[cfg(test)]
mod tests {
    //! Tests for the sample function.
    //!
    //! These tests verify that the sample function creates complete, non-overlapping samples
    //! where each index from 0..n appears exactly once across all samples.
    //!
    //! **Critical test focus**: Floating-point accuracy issues when summing relative sizes.
    //! When relative sizes are converted to actual sample sizes via rounding, the sum of
    //! rounded values may not equal n due to cumulative rounding errors.
    //!
    //! **Solution**: The implementation handles this by giving all remaining indices to the
    //! last sample, ensuring that cumulative rounding errors don't leave any indices unsampled.

    use super::*;
    use std::collections::HashSet;

    /// Helper function to verify that samples are complete and non-overlapping.
    ///
    /// This checks that:
    /// 1. The total number of sampled indices equals n
    /// 2. Each index from 0..n appears exactly once across all samples
    /// 3. No duplicates exist within or across samples
    fn verify_complete_sample(samples: &Vec<Vec<usize>>, n: usize) {
        // Collect all indices from all samples
        let mut all_indices: Vec<usize> = samples.iter().flatten().copied().collect();
        all_indices.sort_unstable();

        // Check that we have exactly n indices
        assert_eq!(all_indices.len(), n, "Total number of sampled indices should be exactly n");

        // Check that each index from 0..n appears exactly once
        for i in 0..n {
            assert_eq!(
                all_indices.iter().filter(|&&x| x == i).count(),
                1,
                "Index {} should appear exactly once, but appears {} times",
                i,
                all_indices.iter().filter(|&&x| x == i).count()
            );
        }

        // Alternative check: verify using a HashSet
        let unique_indices: HashSet<usize> = all_indices.iter().copied().collect();
        assert_eq!(unique_indices.len(), n, "Should have exactly n unique indices");

        // Verify that the union of all indices equals the expected range
        let expected: HashSet<usize> = (0..n).collect();
        assert_eq!(unique_indices, expected, "The set of all sampled indices should equal {{0, 1, ..., n-1}}");
    }

    #[test]
    fn test_simple_equal_split() {
        let sizes = vec![1.0, 1.0];
        let n = 100;
        let samples = sample(&sizes, n, 42);

        verify_complete_sample(&samples, n);
        assert_eq!(samples.len(), 2);
    }

    #[test]
    fn test_unequal_split() {
        let sizes = vec![2.0, 1.0];
        let n = 99;
        let samples = sample(&sizes, n, 42);

        verify_complete_sample(&samples, n);
        assert_eq!(samples.len(), 2);
    }

    #[test]
    fn test_many_small_fractions() {
        // This tests floating-point accuracy with many small divisions
        let sizes = vec![0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1];
        let n = 1000;
        let samples = sample(&sizes, n, 42);

        verify_complete_sample(&samples, n);
        assert_eq!(samples.len(), 10);
    }

    #[test]
    fn test_fractions() {
        // Test with fractions that don't divide evenly
        let sizes = vec![1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0];
        let n = 1000;
        let samples = sample(&sizes, n, 42);

        verify_complete_sample(&samples, n);
        assert_eq!(samples.len(), 3);
    }

    #[test]
    fn test_prime_number_elements() {
        // Prime numbers are good for testing rounding issues
        let sizes = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        let n = 97; // prime number
        let samples = sample(&sizes, n, 42);

        verify_complete_sample(&samples, n);
        assert_eq!(samples.len(), 5);
    }

    #[test]
    fn test_large_number_of_samples() {
        // Many samples with small relative sizes
        let sizes = vec![1.0; 50];
        let n = 1000;
        let samples = sample(&sizes, n, 42);

        verify_complete_sample(&samples, n);
        assert_eq!(samples.len(), 50);
    }

    #[test]
    fn test_very_unequal_distribution() {
        // One large sample and many tiny ones
        let sizes = vec![100.0, 0.01, 0.01, 0.01, 0.01];
        let n = 1000;
        let samples = sample(&sizes, n, 42);

        verify_complete_sample(&samples, n);
        assert_eq!(samples.len(), 5);
    }

    #[test]
    fn test_floating_point_precision() {
        // Test with numbers that have floating-point representation issues
        let sizes = vec![0.1 + 0.2, 0.3, 0.3]; // 0.1 + 0.2 != 0.3 in floating-point
        let n = 300;
        let samples = sample(&sizes, n, 42);

        verify_complete_sample(&samples, n);
        assert_eq!(samples.len(), 3);
    }

    #[test]
    fn test_single_sample() {
        // Edge case: all elements in one sample
        let sizes = vec![1.0];
        let n = 100;
        let samples = sample(&sizes, n, 42);

        verify_complete_sample(&samples, n);
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].len(), n);
    }

    #[test]
    fn test_small_n() {
        // Test with small n where rounding matters more
        let sizes = vec![1.0, 1.0, 1.0];
        let n = 10;
        let samples = sample(&sizes, n, 42);

        verify_complete_sample(&samples, n);
        assert_eq!(samples.len(), 3);
    }

    #[test]
    fn test_n_smaller_than_samples() {
        // When n is smaller than the number of sample groups
        let sizes = vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let n = 5;
        let samples = sample(&sizes, n, 42);

        verify_complete_sample(&samples, n);
        assert_eq!(samples.len(), 8);
    }

    #[test]
    fn test_deterministic_with_same_seed() {
        // Test that same seed produces same results
        let sizes = vec![1.0, 1.0, 1.0];
        let n = 100;

        let samples1 = sample(&sizes, n, 12345);
        let samples2 = sample(&sizes, n, 12345);

        verify_complete_sample(&samples1, n);
        verify_complete_sample(&samples2, n);

        // Should produce identical results
        assert_eq!(samples1, samples2);
    }

    #[test]
    fn test_different_seeds_different_results() {
        // Test that different seeds produce different results
        let sizes = vec![1.0, 1.0, 1.0];
        let n = 100;

        let samples1 = sample(&sizes, n, 1);
        let samples2 = sample(&sizes, n, 2);

        verify_complete_sample(&samples1, n);
        verify_complete_sample(&samples2, n);

        // Should produce different results (not a guarantee but very likely)
        assert_ne!(samples1, samples2);
    }

    #[test]
    fn test_sum_of_fractions_with_cumulative_error() {
        // Test a case where cumulative rounding errors might accumulate
        // Using values that when summed and divided create rounding issues
        let sizes = vec![0.3333, 0.3333, 0.3334]; // These don't sum to exactly 1.0
        let n = 1000;
        let samples = sample(&sizes, n, 42);

        verify_complete_sample(&samples, n);
        assert_eq!(samples.len(), 3);
    }

    #[test]
    fn test_very_small_relative_sizes() {
        // Test with very small numbers that might cause precision issues
        let sizes = vec![1e-10, 1e-10, 1e-10, 1e-10];
        let n = 100;
        let samples = sample(&sizes, n, 42);

        verify_complete_sample(&samples, n);
        assert_eq!(samples.len(), 4);
    }
}
