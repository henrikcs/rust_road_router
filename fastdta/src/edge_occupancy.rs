use rust_road_router::datastr::graph::floating_time_dependent::TDGraph;
use rust_road_router::datastr::graph::{
    Graph,
    floating_time_dependent::{FlWeight, Timestamp},
};

/// given a graph, a set of old paths, a set of new paths, and their respective departure times,
/// compute the edge occupancy deltas for each edge in the graph over time
/// the occupancy is sum of travel times of all vehicles on the edge within a time window,
/// where the occupancy for edges along old paths is subtracted
/// and the occupancy for edges along new paths is added
/// the period is given in seconds
/// the path is given as a vector of edge ids
/// returns a vector of vectors, where the outer vector is indexed by period
/// and the inner vector is indexed by edge id
pub fn get_edge_occupancy_deltas<G: TravelTimeGraph>(
    graph: &G,
    old_paths: &Vec<Vec<u32>>,
    new_paths: &Vec<Vec<u32>>,
    departures: &Vec<Timestamp>,
    periods: &Vec<(f64, f64)>,
) -> Vec<Vec<f64>> {
    // Debug assertion: verify periods have no holes (consecutive periods are continuous)
    debug_assert!(periods.windows(2).all(|w| w[0].1 == w[1].0), "Periods must be continuous with no gaps");

    let num_edges = graph.num_arcs();
    let mut edge_occupancy_deltas = vec![vec![0.0; num_edges]; periods.len()];

    // Process old paths (subtract travel times)
    for (path_idx, path) in old_paths.iter().enumerate() {
        if path_idx < departures.len() {
            process_path(graph, path, departures[path_idx], -1.0, periods, &mut edge_occupancy_deltas);
        }
    }

    // Process new paths (add travel times)
    for (path_idx, path) in new_paths.iter().enumerate() {
        if path_idx < departures.len() {
            process_path(graph, path, departures[path_idx], 1.0, periods, &mut edge_occupancy_deltas);
        }
    }

    edge_occupancy_deltas
}

/// Trait for graphs that can provide travel time calculations
/// This allows for mocking in tests
pub trait TravelTimeGraph {
    fn num_arcs(&self) -> usize;
    fn get_travel_time_along_path(&self, departure_time: Timestamp, path: &[u32]) -> FlWeight;
}

/// Implementation of TravelTimeGraph for TDGraph
impl TravelTimeGraph for TDGraph {
    fn num_arcs(&self) -> usize {
        Graph::num_arcs(self)
    }

    fn get_travel_time_along_path(&self, departure_time: Timestamp, path: &[u32]) -> FlWeight {
        TDGraph::get_travel_time_along_path(&self, departure_time, path)
    }
}

/// Process a single path and update edge occupancy deltas
fn process_path<G: TravelTimeGraph>(
    graph: &G,
    path: &Vec<u32>,
    departure_time: Timestamp,
    sign: f64,
    periods: &Vec<(f64, f64)>,
    edge_occupancy_deltas: &mut Vec<Vec<f64>>,
) {
    let mut current_time = departure_time;
    for &edge_id in path {
        let travel_time = graph.get_travel_time_along_path(current_time, &[edge_id]);
        // dbg!(f64::from(travel_time));
        // println!("Edge ID: {}, Travel Time: {:?}", edge_id, travel_time);
        let arrival_time = current_time + travel_time;
        // dbg!(arrival_time);
        let bin_search_res = match periods.binary_search_by(|(start, end)| {
            // if current time is between start and end: return equal:
            if *start <= f64::from(current_time) && f64::from(current_time) < *end {
                std::cmp::Ordering::Equal
            } else if f64::from(current_time) < *start {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            }
        }) {
            Ok(idx) => idx,
            Err(idx) => {
                println!("Binary search for current_time {} returned Err({})", f64::from(current_time), idx);
                0
            }
        };

        for (period_idx, &(period_start, period_end)) in periods.iter().skip(bin_search_res).enumerate() {
            if current_time >= arrival_time {
                break; // No more travel time to distribute
            }

            // Skip periods that end before our current time
            if period_end <= f64::from(current_time) {
                continue;
            }

            // Skip periods that start after our travel ends
            if period_start >= f64::from(arrival_time) {
                break;
            }

            // Calculate the overlap between travel time and this period
            let overlap_start = f64::from(current_time).max(period_start);
            let overlap_end = f64::from(arrival_time).min(period_end);
            let overlap_duration = overlap_end - overlap_start;

            if overlap_duration > 0.0 {
                edge_occupancy_deltas[bin_search_res + period_idx][edge_id as usize] += sign * overlap_duration;

                if period_start == 300.0 && (edge_id == 6 || edge_id == 8) {
                    // println!(
                    //     "Edge ID: {}, Period: {}-{}, Overlap: {}, Sign: {}, Updated Delta: {}",
                    //     edge_id,
                    //     period_start,
                    //     period_end,
                    //     overlap_duration,
                    //     sign,
                    //     edge_occupancy_deltas[bin_search_res + period_idx][edge_id as usize]
                    // );
                }
            }
            // Move to the next period boundary for the next iteration
            current_time = Timestamp::new(overlap_end);
            // dbg!(current_time);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Mock implementation of TravelTimeGraph for testing
    struct MockTravelTimeGraph {
        num_edges: usize,
        // Map from edge_id to travel time
        travel_times: HashMap<u32, FlWeight>,
    }

    impl MockTravelTimeGraph {
        fn new(num_edges: usize) -> Self {
            Self {
                num_edges,
                travel_times: HashMap::new(),
            }
        }

        fn set_travel_time(&mut self, edge_id: u32, travel_time: FlWeight) {
            self.travel_times.insert(edge_id, travel_time);
        }
    }

    impl TravelTimeGraph for MockTravelTimeGraph {
        fn num_arcs(&self) -> usize {
            self.num_edges
        }

        fn get_travel_time_along_path(&self, _departure_time: Timestamp, path: &[u32]) -> FlWeight {
            // Return the travel time for the first edge in path (since we're testing single-edge paths)
            if let Some(&edge_id) = path.first() {
                self.travel_times.get(&edge_id).copied().unwrap_or(FlWeight::new(10.0))
            } else {
                FlWeight::ZERO
            }
        }
    }

    #[test]
    fn test_same_paths_zero_deltas() {
        // Test case 1: old_paths = new_paths -> all deltas should be zero
        let mut mock_graph = MockTravelTimeGraph::new(3);
        mock_graph.set_travel_time(0, FlWeight::new(5.0));
        mock_graph.set_travel_time(1, FlWeight::new(3.0));
        mock_graph.set_travel_time(2, FlWeight::new(7.0));

        let old_paths = vec![vec![0], vec![1], vec![2]];
        let new_paths = vec![vec![0], vec![1], vec![2]]; // Same as old_paths
        let departures = vec![Timestamp::new(0.0), Timestamp::new(10.0), Timestamp::new(20.0)];
        let periods = vec![(0.0, 10.0), (10.0, 20.0), (20.0, 30.0)];

        let result = get_edge_occupancy_deltas(&mock_graph, &old_paths, &new_paths, &departures, &periods);

        // All deltas should be zero since we subtract and add the same values
        for period_deltas in result {
            for delta in period_deltas {
                assert!((delta.abs()) < 1e-10, "Expected zero delta, got {}", delta);
            }
        }
    }

    #[test]
    fn test_different_paths_nonzero_deltas() {
        // Test case 2: old_paths != new_paths -> deltas sum up accordingly
        let mut mock_graph = MockTravelTimeGraph::new(3);
        mock_graph.set_travel_time(0, FlWeight::new(5.0));
        mock_graph.set_travel_time(1, FlWeight::new(3.0));
        mock_graph.set_travel_time(2, FlWeight::new(7.0));

        let old_paths = vec![vec![0]]; // Single path using edge 0
        let new_paths = vec![vec![1]]; // Single path using edge 1
        let departures = vec![Timestamp::new(0.0)];
        let periods = vec![(0.0, 10.0)];

        let result = get_edge_occupancy_deltas(&mock_graph, &old_paths, &new_paths, &departures, &periods);

        // Edge 0 should have negative delta (old path removed)
        assert!(result[0][0] < 0.0, "Edge 0 should have negative delta");
        // Edge 1 should have positive delta (new path added)
        assert!(result[0][1] > 0.0, "Edge 1 should have positive delta");
        // Edge 2 should have zero delta (not used)
        assert!((result[0][2].abs()) < 1e-10, "Edge 2 should have zero delta");
    }

    #[test]
    #[should_panic(expected = "Periods must be continuous with no gaps")]
    fn test_periods_with_holes_panic() {
        // Test case 3: periods contain holes -> expect an assertion failure
        let mock_graph = MockTravelTimeGraph::new(1);
        let old_paths = vec![vec![0]];
        let new_paths = vec![vec![0]];
        let departures = vec![Timestamp::new(0.0)];

        // Periods with a gap between them
        let periods = vec![(0.0, 10.0), (15.0, 25.0)]; // Gap from 10.0 to 15.0

        // This should panic due to the assertion
        get_edge_occupancy_deltas(&mock_graph, &old_paths, &new_paths, &departures, &periods);
    }

    #[test]
    fn test_travel_time_overlap_calculation() {
        // Test that travel time overlaps with periods are calculated correctly
        let mut mock_graph = MockTravelTimeGraph::new(1);
        mock_graph.set_travel_time(0, FlWeight::new(15.0)); // Travel time spans multiple periods

        let old_paths = vec![];
        let new_paths = vec![vec![0]];
        let departures = vec![Timestamp::new(5.0)]; // Depart at t=5, arrive at t=20
        let periods = vec![(0.0, 10.0), (10.0, 20.0), (20.0, 30.0)];

        let result = get_edge_occupancy_deltas(&mock_graph, &old_paths, &new_paths, &departures, &periods);

        // Edge 0 should have positive deltas in first two periods due to overlap
        assert!(result[0][0] > 0.0, "Period 0 should have positive delta");
        assert!(result[1][0] > 0.0, "Period 1 should have positive delta");
        assert!((result[2][0].abs()) < 1e-10, "Period 2 should have zero delta");

        // The total delta should equal the travel time
        let total_delta = result[0][0] + result[1][0] + result[2][0];
        assert!((total_delta - 15.0).abs() < 1e-10, "Total delta should equal travel time");
    }

    #[test]
    fn test_travel_time_divided_between_two_periods() {
        // Test case: travel time is divided between exactly two periods
        let mut mock_graph = MockTravelTimeGraph::new(1);
        mock_graph.set_travel_time(0, FlWeight::new(6.0)); // Travel time: 6 seconds

        let old_paths = vec![];
        let new_paths = vec![vec![0]];
        let departures = vec![Timestamp::new(7.0)]; // Depart at t=7, arrive at t=13
        let periods = vec![(0.0, 10.0), (10.0, 20.0), (20.0, 30.0)];

        let result = get_edge_occupancy_deltas(&mock_graph, &old_paths, &new_paths, &departures, &periods);

        // Expected distribution:
        // Period 0 (0-10): overlap from t=7 to t=10 = 3 seconds
        // Period 1 (10-20): overlap from t=10 to t=13 = 3 seconds
        // Period 2 (20-30): no overlap = 0 seconds

        assert!((result[0][0] - 3.0).abs() < 1e-10, "Period 0 should have 3.0 delta, got {}", result[0][0]);
        assert!((result[1][0] - 3.0).abs() < 1e-10, "Period 1 should have 3.0 delta, got {}", result[1][0]);
        assert!((result[2][0].abs()) < 1e-10, "Period 2 should have zero delta, got {}", result[2][0]);

        // Total should equal travel time
        let total_delta = result[0][0] + result[1][0] + result[2][0];
        assert!(
            (total_delta - 6.0).abs() < 1e-10,
            "Total delta should equal travel time of 6.0, got {}",
            total_delta
        );
    }

    #[test]
    fn test_travel_time_divided_between_three_periods() {
        // Test case: travel time is divided between exactly three periods
        let mut mock_graph = MockTravelTimeGraph::new(1);
        mock_graph.set_travel_time(0, FlWeight::new(18.0)); // Travel time: 18 seconds

        let old_paths = vec![];
        let new_paths = vec![vec![0]];
        let departures = vec![Timestamp::new(4.0)]; // Depart at t=4, arrive at t=22
        let periods = vec![(0.0, 10.0), (10.0, 15.0), (15.0, 25.0), (25.0, 35.0)];

        let result = get_edge_occupancy_deltas(&mock_graph, &old_paths, &new_paths, &departures, &periods);

        // Expected distribution:
        // Period 0 (0-10): overlap from t=4 to t=10 = 6 seconds
        // Period 1 (10-15): overlap from t=10 to t=15 = 5 seconds
        // Period 2 (15-25): overlap from t=15 to t=22 = 7 seconds
        // Period 3 (25-35): no overlap = 0 seconds

        assert!((result[0][0] - 6.0).abs() < 1e-10, "Period 0 should have 6.0 delta, got {}", result[0][0]);
        assert!((result[1][0] - 5.0).abs() < 1e-10, "Period 1 should have 5.0 delta, got {}", result[1][0]);
        assert!((result[2][0] - 7.0).abs() < 1e-10, "Period 2 should have 7.0 delta, got {}", result[2][0]);
        assert!((result[3][0].abs()) < 1e-10, "Period 3 should have zero delta, got {}", result[3][0]);

        // Total should equal travel time
        let total_delta = result[0][0] + result[1][0] + result[2][0] + result[3][0];
        assert!(
            (total_delta - 18.0).abs() < 1e-10,
            "Total delta should equal travel time of 18.0, got {}",
            total_delta
        );
    }
}
