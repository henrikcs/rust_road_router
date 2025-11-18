use std::collections::HashMap;

use conversion::SUMO_MAX_TRAVEL_TIME;
use conversion::sumo::meandata::Interval;
use rust_road_router::datastr::graph::floating_time_dependent::{TDGraph, TTFPoint};
use rust_road_router::datastr::graph::{
    Graph,
    floating_time_dependent::{FlWeight, Timestamp},
};

use crate::traffic_model::TrafficModel;

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
    graph: &mut G,
    old_paths: &Vec<&Vec<u32>>,
    new_paths: &Vec<Vec<u32>>,
    departures: &Vec<Timestamp>,
    intervals: &mut Vec<Interval>,
    edge_ids: &Vec<String>,
    edge_lengths: &Vec<f64>,
    edge_free_flow_tts: &Vec<f64>,
    traffic_model: &HashMap<usize, Box<dyn TrafficModel>>,
    lanes: &Vec<u32>,
) -> Vec<Vec<f64>> {
    // Debug assertion: verify periods have no holes (consecutive periods are continuous)
    debug_assert!(intervals.windows(2).all(|w| w[0].end == w[1].begin), "Periods must be continuous with no gaps");

    // dbg!(&graph.ipps());

    let num_edges = graph.num_arcs();
    let mut edge_occupancy_deltas = vec![vec![0.0; num_edges]; intervals.len()];

    // Process old paths (subtract travel times)
    for (path_idx, path) in old_paths.iter().enumerate() {
        process_path(
            graph,
            path,
            departures[path_idx],
            -1.0,
            intervals,
            &mut edge_occupancy_deltas,
            edge_ids,
            edge_lengths,
            edge_free_flow_tts,
            traffic_model,
            lanes,
        );
    }

    // Process new paths (add travel times)
    for (path_idx, path) in new_paths.iter().enumerate() {
        process_path(
            graph,
            path,
            departures[path_idx],
            1.0,
            intervals,
            &mut edge_occupancy_deltas,
            edge_ids,
            edge_lengths,
            edge_free_flow_tts,
            traffic_model,
            lanes,
        );
    }

    edge_occupancy_deltas
}

/// Trait for graphs that can provide travel time calculations
/// This allows for mocking in tests
pub trait TravelTimeGraph {
    fn num_arcs(&self) -> usize;
    fn get_travel_time_along_path(&self, departure_time: Timestamp, path: &[u32]) -> FlWeight;
    fn set_weight_for_edge_at_time(&mut self, edge_id: u32, at: Timestamp, new_weight: FlWeight);
    fn ipps(&self) -> &Vec<TTFPoint>;
}

/// Implementation of TravelTimeGraph for TDGraph
impl TravelTimeGraph for TDGraph {
    fn num_arcs(&self) -> usize {
        Graph::num_arcs(self)
    }

    fn get_travel_time_along_path(&self, departure_time: Timestamp, path: &[u32]) -> FlWeight {
        TDGraph::get_travel_time_along_path(&self, departure_time, path)
    }

    fn set_weight_for_edge_at_time(&mut self, edge_id: u32, at: Timestamp, new_weight: FlWeight) {
        TDGraph::set_weight_for_edge_at_time(self, edge_id, at, new_weight);
    }

    fn ipps(&self) -> &Vec<TTFPoint> {
        &self.ipps
    }
}

/// Process a single path and update edge occupancy deltas
fn process_path<G: TravelTimeGraph>(
    graph: &mut G,
    path: &Vec<u32>,
    departure_time: Timestamp,
    sign: f64,
    intervals: &mut Vec<Interval>,
    edge_occupancy_deltas: &mut Vec<Vec<f64>>,
    edge_ids: &Vec<String>,
    edge_lengths: &Vec<f64>,
    edge_free_flow_tts: &Vec<f64>,
    traffic_model: &HashMap<usize, Box<dyn TrafficModel>>,
    lanes: &Vec<u32>,
) {
    let mut current_time = departure_time;
    for &edge_id in path {
        let travel_time = graph.get_travel_time_along_path(current_time, &[edge_id]);
        // dbg!(f64::from(travel_time));
        // println!("Edge ID: {}, Travel Time: {:?}", edge_id, travel_time);
        let arrival_time = current_time + travel_time;
        // dbg!(arrival_time);
        let bin_search_res = match intervals.binary_search_by(|interval| {
            // if current time is between start and end: return equal:
            if interval.begin <= f64::from(current_time) && f64::from(current_time) < interval.end {
                std::cmp::Ordering::Equal
            } else if f64::from(current_time) < interval.begin {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            }
        }) {
            Ok(idx) => idx,
            Err(_) => {
                // no more periods to process
                // println!("Binary search for current_time {} returned Err({})", f64::from(current_time), idx);
                return;
            }
        };

        for (interval_idx, interval) in intervals.iter_mut().skip(bin_search_res).enumerate() {
            if current_time >= arrival_time {
                break; // No more travel time to distribute
            }

            // Skip periods that end before or at our current time
            if interval.end <= f64::from(current_time) {
                continue;
            }

            // Skip periods that start after or at our travel ends
            if interval.begin >= f64::from(arrival_time) {
                break;
            }

            // Calculate the overlap between travel time and this period
            let overlap_start = f64::from(current_time).max(interval.begin);
            let overlap_end = f64::from(arrival_time).min(interval.end);
            let overlap_duration = overlap_end - overlap_start;

            if overlap_duration > 0.0 {
                edge_occupancy_deltas[bin_search_res + interval_idx][edge_id as usize] += sign * overlap_duration;

                let interval_duration = interval.end - interval.begin;
                let interval_begin = interval.begin;
                interval.get_edge_mut(edge_ids[edge_id as usize].as_str()).map(|edge| {
                    let previous_sampled = edge.sampled_seconds.unwrap_or(0.0);
                    let previous_tt = edge.traveltime;
                    let previous_density = edge.lane_density.unwrap_or(edge.density.unwrap_or(0.0));
                    let previous_estimated_density = edge.get_lane_density(interval_duration, edge_lengths[edge_id as usize], lanes[edge_id as usize]);
                    edge.sampled_seconds = Some(f64::max(edge.sampled_seconds.unwrap_or(0.0) + sign * overlap_duration, 0.0));

                    let estimated_density = edge.get_lane_density(interval_duration, edge_lengths[edge_id as usize], lanes[edge_id as usize]);

                    let estimated_tt = traffic_model.get(&(edge_id as usize)).map_or(edge_free_flow_tts[edge_id as usize], |tm| {
                        let tt = edge_lengths[edge_id as usize] / tm.get_speed(estimated_density) / 3.6;
                        if tt < 0.0 {
                            return SUMO_MAX_TRAVEL_TIME;
                        }
                        tt
                    });

                    if edge_ids[edge_id as usize] == "a2" && interval_begin == 1550.0 {
                        println!(
                            "Update edge {} (l={}) at time {}: sampled_seconds: {:?} -> {:?}, traveltime: {:?} -> {:?}, density: {} (est.: {}) -> {}",
                            edge_ids[edge_id as usize],
                            edge_lengths[edge_id as usize],
                            interval_begin,
                            previous_sampled,
                            edge.sampled_seconds,
                            previous_tt,
                            estimated_tt,
                            previous_density,
                            previous_estimated_density,
                            estimated_density
                        );
                        if estimated_tt < 0.0 {
                            let tm = traffic_model.get(&(edge_id as usize)).unwrap();
                            tm.debug();
                            panic!("Estimated travel time for edge {} is negative: {}", edge_ids[edge_id as usize], estimated_tt);
                        }
                    }

                    graph.set_weight_for_edge_at_time(edge_id, Timestamp::new(interval_begin), FlWeight::new(estimated_tt));
                    edge.traveltime = Some(estimated_tt);
                });
            }
            // Move to the next period boundary for the next iteration
            current_time = Timestamp::new(overlap_end);
            // dbg!(current_time);
        }
    }
}

/*
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

        fn set_weight_for_edge_at_time(&mut self, _edge_id: u32, _at: Timestamp, _new_weight: FlWeight) {
            // No-op for mock
        }

        fn ipps(&self) -> &Vec<TTFPoint> {
            panic!();
        }
    }

    #[test]
    fn test_same_paths_zero_deltas() {
        // Test case 1: old_paths = new_paths -> all deltas should be zero
        let mut mock_graph = MockTravelTimeGraph::new(3);
        mock_graph.set_travel_time(0, FlWeight::new(5.0));
        mock_graph.set_travel_time(1, FlWeight::new(3.0));
        mock_graph.set_travel_time(2, FlWeight::new(7.0));

        let old_paths_owned = vec![vec![0], vec![1], vec![2]];
        let old_paths: Vec<&Vec<u32>> = old_paths_owned.iter().collect();
        let new_paths = vec![vec![0], vec![1], vec![2]]; // Same as old_paths
        let departures = vec![Timestamp::new(0.0), Timestamp::new(10.0), Timestamp::new(20.0)];
        let free_flow_tts = vec![0.0, 0.0, 0.0];
        let edge_lanes = vec![1, 1, 1];
        let mut intervals = vec![
            Interval::create("0".to_string(), 0.0, 10.0, vec![]),
            Interval::create("1".to_string(), 10.0, 20.0, vec![]),
            Interval::create("2".to_string(), 20.0, 30.0, vec![]),
        ];
        let edge_ids = vec!["edge0".to_string(), "edge1".to_string(), "edge2".to_string()];
        let edge_lengths = vec![100.0, 150.0, 200.0];

        let result = get_edge_occupancy_deltas(
            &mut mock_graph,
            &old_paths,
            &new_paths,
            &departures,
            &mut intervals,
            &edge_ids,
            &edge_lengths,
            &free_flow_tts,
            &edge_lanes,
        );

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

        let old_paths_owned = vec![vec![0]]; // Single path using edge 0
        let old_paths: Vec<&Vec<u32>> = old_paths_owned.iter().collect();
        let new_paths = vec![vec![1]]; // Single path using edge 1
        let departures = vec![Timestamp::new(0.0)];

        // Create edges for the interval
        use conversion::sumo::meandata::Edge;
        let edges = vec![
            Edge {
                id: "edge0".to_string(),
                traveltime: None,
                density: None,
                speed: None,
                sampled_seconds: Some(0.0),
                lanedensity: None,
            },
            Edge {
                id: "edge1".to_string(),
                traveltime: None,
                density: None,
                speed: None,
                sampled_seconds: Some(0.0),
                lanedensity: None,
            },
            Edge {
                id: "edge2".to_string(),
                traveltime: None,
                density: None,
                speed: None,
                sampled_seconds: Some(0.0),
                lanedensity: None,
            },
        ];
        let free_flow_tts = vec![0.0, 0.0, 0.0];
        let mut intervals = vec![Interval::create("0".to_string(), 0.0, 10.0, edges)];
        let edge_ids = vec!["edge0".to_string(), "edge1".to_string(), "edge2".to_string()];
        let edge_lengths = vec![100.0, 150.0, 200.0];
        let edge_lanes = vec![1, 1, 1];

        let result = get_edge_occupancy_deltas(
            &mut mock_graph,
            &old_paths,
            &new_paths,
            &departures,
            &mut intervals,
            &edge_ids,
            &edge_lengths,
            &free_flow_tts,
            &edge_lanes,
        );

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
        let mut mock_graph = MockTravelTimeGraph::new(1);
        let old_paths_owned = vec![vec![0]];
        let old_paths: Vec<&Vec<u32>> = old_paths_owned.iter().collect();
        let new_paths = vec![vec![0]];
        let departures = vec![Timestamp::new(0.0)];

        let free_flow_tts = vec![0.0, 0.0, 0.0];
        // Periods with a gap between them
        let mut intervals = vec![
            Interval::create("0".to_string(), 0.0, 10.0, vec![]),
            Interval::create("1".to_string(), 15.0, 25.0, vec![]), // Gap from 10.0 to 15.0
        ];
        let edge_ids = vec!["edge0".to_string()];
        let edge_lengths = vec![100.0];
        let edge_lanes = vec![1];

        // This should panic due to the assertion
        get_edge_occupancy_deltas(
            &mut mock_graph,
            &old_paths,
            &new_paths,
            &departures,
            &mut intervals,
            &edge_ids,
            &edge_lengths,
            &free_flow_tts,
            &edge_lanes,
        );
    }

    #[test]
    fn test_travel_time_overlap_calculation() {
        // Test that travel time overlaps with periods are calculated correctly
        let mut mock_graph = MockTravelTimeGraph::new(1);
        mock_graph.set_travel_time(0, FlWeight::new(15.0)); // Travel time spans multiple periods

        let old_paths_owned = vec![];
        let old_paths: Vec<&Vec<u32>> = old_paths_owned.iter().collect();
        let new_paths = vec![vec![0]];
        let departures = vec![Timestamp::new(5.0)]; // Depart at t=5, arrive at t=20

        // Create edges for each interval
        use conversion::sumo::meandata::Edge;
        let edges0 = vec![Edge {
            id: "edge0".to_string(),
            traveltime: None,
            density: None,
            speed: None,
            sampled_seconds: Some(0.0),
            lanedensity: None,
        }];
        let edges1 = vec![Edge {
            id: "edge0".to_string(),
            traveltime: None,
            density: None,
            speed: None,
            sampled_seconds: Some(0.0),
            lanedensity: None,
        }];
        let edges2 = vec![Edge {
            id: "edge0".to_string(),
            traveltime: None,
            density: None,
            speed: None,
            sampled_seconds: Some(0.0),
            lanedensity: None,
        }];

        let mut intervals = vec![
            Interval::create("0".to_string(), 0.0, 10.0, edges0),
            Interval::create("1".to_string(), 10.0, 20.0, edges1),
            Interval::create("2".to_string(), 20.0, 30.0, edges2),
        ];
        let edge_ids = vec!["edge0".to_string()];
        let edge_lengths = vec![100.0];
        let free_flow_tts = vec![0.0, 0.0, 0.0];

        let edge_lanes = vec![1];

        let result = get_edge_occupancy_deltas(
            &mut mock_graph,
            &old_paths,
            &new_paths,
            &departures,
            &mut intervals,
            &edge_ids,
            &edge_lengths,
            &free_flow_tts,
            &edge_lanes,
        );

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

        let old_paths_owned = vec![];
        let old_paths: Vec<&Vec<u32>> = old_paths_owned.iter().collect();
        let new_paths = vec![vec![0]];
        let departures = vec![Timestamp::new(7.0)]; // Depart at t=7, arrive at t=13

        // Create edges for each interval
        use conversion::sumo::meandata::Edge;
        let edges0 = vec![Edge {
            id: "edge0".to_string(),
            traveltime: None,
            density: None,
            speed: None,
            sampled_seconds: Some(0.0),
            lanedensity: None,
        }];
        let edges1 = vec![Edge {
            id: "edge0".to_string(),
            traveltime: None,
            density: None,
            speed: None,
            sampled_seconds: Some(0.0),
            lanedensity: None,
        }];
        let edges2 = vec![Edge {
            id: "edge0".to_string(),
            traveltime: None,
            density: None,
            speed: None,
            sampled_seconds: Some(0.0),
            lanedensity: None,
        }];

        let free_flow_tts = vec![0.0, 0.0, 0.0];
        let mut intervals = vec![
            Interval::create("0".to_string(), 0.0, 10.0, edges0),
            Interval::create("1".to_string(), 10.0, 20.0, edges1),
            Interval::create("2".to_string(), 20.0, 30.0, edges2),
        ];
        let edge_ids = vec!["edge0".to_string()];
        let edge_lengths = vec![100.0];
        let edge_lanes = vec![1];

        let result = get_edge_occupancy_deltas(
            &mut mock_graph,
            &old_paths,
            &new_paths,
            &departures,
            &mut intervals,
            &edge_ids,
            &edge_lengths,
            &free_flow_tts,
            &edge_lanes,
        );

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

        let old_paths_owned = vec![];
        let old_paths: Vec<&Vec<u32>> = old_paths_owned.iter().collect();
        let new_paths = vec![vec![0]];
        let departures = vec![Timestamp::new(4.0)]; // Depart at t=4, arrive at t=22

        // Create edges for each interval
        use conversion::sumo::meandata::Edge;
        let edges0 = vec![Edge {
            id: "edge0".to_string(),
            traveltime: None,
            density: None,
            speed: None,
            sampled_seconds: Some(0.0),
            lanedensity: None,
        }];
        let edges1 = vec![Edge {
            id: "edge0".to_string(),
            traveltime: None,
            density: None,
            speed: None,
            sampled_seconds: Some(0.0),
            lanedensity: None,
        }];
        let edges2 = vec![Edge {
            id: "edge0".to_string(),
            traveltime: None,
            density: None,
            speed: None,
            sampled_seconds: Some(0.0),
            lanedensity: None,
        }];
        let edges3 = vec![Edge {
            id: "edge0".to_string(),
            traveltime: None,
            density: None,
            speed: None,
            sampled_seconds: Some(0.0),
            lanedensity: None,
        }];

        let free_flow_tts = vec![0.0, 0.0, 0.0];
        let mut intervals = vec![
            Interval::create("0".to_string(), 0.0, 10.0, edges0),
            Interval::create("1".to_string(), 10.0, 15.0, edges1),
            Interval::create("2".to_string(), 15.0, 25.0, edges2),
            Interval::create("3".to_string(), 25.0, 35.0, edges3),
        ];
        let edge_ids = vec!["edge0".to_string()];
        let edge_lengths = vec![100.0];
        let edge_lanes = vec![1];

        let result = get_edge_occupancy_deltas(
            &mut mock_graph,
            &old_paths,
            &new_paths,
            &departures,
            &mut intervals,
            &edge_ids,
            &edge_lengths,
            &free_flow_tts,
            &edge_lanes,
        );

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
*/
