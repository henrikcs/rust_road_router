use conversion::SUMO_MAX_TRAVEL_TIME;
use conversion::sumo::meandata::{Edge, Interval};
use rust_road_router::datastr::graph::floating_time_dependent::{EPSILON, TDGraph, TTFPoint};
use rust_road_router::datastr::graph::{
    Graph,
    floating_time_dependent::{FlWeight, Timestamp},
};

use crate::traffic_model::TrafficModel;

/// given a graph, a set of old paths, a set of new paths, and their respective departure times,
/// computes the estimated densities on the edges which vehicles are rerouted on
/// and updates the graph's travel times accordingly.
pub fn adjust_weights_in_graph_by_following_paths<G: TravelTimeGraph>(
    graph: &mut G,
    old_paths: &Vec<&Vec<u32>>,
    new_paths: &Vec<Vec<u32>>,
    departures: &Vec<Timestamp>,
    intervals: &mut Vec<Interval>,
    edge_ids: &Vec<String>,
    edge_lengths: &Vec<f64>,
    edge_free_flow_tts: &Vec<f64>,
    traffic_models: &Vec<Box<dyn TrafficModel>>,
    lanes: &Vec<u32>,
) {
    // Debug assertion: verify periods have no holes (consecutive periods are continuous)
    debug_assert!(intervals.windows(2).all(|w| w[0].end == w[1].begin), "Periods must be continuous with no gaps");

    // dbg!(&graph.ipps());

    // Process old paths (subtract travel times)
    for (path_idx, path) in old_paths.iter().enumerate() {
        process_path(
            graph,
            path,
            departures[path_idx],
            -1.0,
            intervals,
            edge_ids,
            edge_lengths,
            edge_free_flow_tts,
            traffic_models,
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
            edge_ids,
            edge_lengths,
            edge_free_flow_tts,
            traffic_models,
            lanes,
        );
    }
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
    edge_ids: &Vec<String>,
    edge_lengths: &Vec<f64>,
    edge_free_flow_tts: &Vec<f64>,
    traffic_models: &Vec<Box<dyn TrafficModel>>,
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

        if let Some(interval) = intervals.get_mut(bin_search_res) {
            // add edge, if not present
            if interval.get_edge(edge_ids[edge_id as usize].as_str()).is_none() {
                interval.add_edge(Edge {
                    id: edge_ids[edge_id as usize].clone(),
                    sampled_seconds: Some(0.0),
                    ..Default::default()
                });
            }

            // set departed to edge.departed = edge.departed + sign * 1;
            if let Some(edge) = interval.get_edge_mut(edge_ids[edge_id as usize].as_str()) {
                edge.dbg_entered = edge.dbg_entered + (sign as u32);
            }
        }

        for (_interval_idx, interval) in intervals.iter_mut().skip(bin_search_res).enumerate() {
            // Skip periods that start after or at our travel ends
            if current_time >= arrival_time || interval.begin >= f64::from(arrival_time) {
                if let Some(interval) = intervals.get_mut(bin_search_res) {
                    // set departed to edge.departed = edge.departed + sign * 1;
                    if let Some(edge) = interval.get_edge_mut(edge_ids[edge_id as usize].as_str()) {
                        edge.dbg_left = edge.dbg_left + (sign as u32);
                    }
                }
                break; // No more travel time to distribute
            }

            // Calculate the overlap between travel time and this period
            let overlap_start = f64::from(current_time).max(interval.begin);
            let overlap_end = f64::from(arrival_time).min(interval.end);
            let overlap_duration = overlap_end - overlap_start;

            if overlap_duration > 0.0 {
                let interval_duration = interval.end - interval.begin;
                let interval_begin = interval.begin;

                interval.get_edge_mut(edge_ids[edge_id as usize].as_str()).map(|edge| {
                    // let previous_sampled = edge.sampled_seconds.unwrap_or(0.0);
                    // let previous_tt = edge.overlap_traveltime;
                    // let previous_density = edge.lane_density.unwrap_or(edge.density.unwrap_or(0.0));
                    // let previous_estimated_density = edge.get_lane_density(interval_duration, edge_lengths[edge_id as usize], lanes[edge_id as usize]);
                    edge.sampled_seconds = Some(f64::max(edge.sampled_seconds.unwrap_or(0.0) + sign * overlap_duration, 0.0));

                    let estimated_density = edge.get_lane_density(interval_duration, edge_lengths[edge_id as usize], lanes[edge_id as usize]);

                    edge.lane_density = Some(estimated_density);

                    let estimated_tt = traffic_models.get(edge_id as usize).map_or(edge_free_flow_tts[edge_id as usize], |tm| {
                        let speed = tm.get_speed(estimated_density) / 3.6;

                        if speed.abs() < EPSILON {
                            return SUMO_MAX_TRAVEL_TIME;
                        }

                        edge.speed = Some(speed);
                        let tt = edge_lengths[edge_id as usize] / speed;
                        if tt < 0.0 {
                            return SUMO_MAX_TRAVEL_TIME;
                        }
                        f64::min(tt, SUMO_MAX_TRAVEL_TIME)
                    });

                    // if edge_ids[edge_id as usize] == "a2" && interval_begin == 1550.0 {
                    //     println!(
                    //         "Update edge {} (l={}) at time {}: sampled_seconds: {:?} -> {:?}, traveltime: {:?} -> {:?}, density: {} (est.: {}) -> {}",
                    //         edge_ids[edge_id as usize],
                    //         edge_lengths[edge_id as usize],
                    //         interval_begin,
                    //         previous_sampled,
                    //         edge.sampled_seconds,
                    //         previous_tt,
                    //         estimated_tt,
                    //         previous_density,
                    //         previous_estimated_density,
                    //         estimated_density
                    //     );
                    //     if estimated_tt < 0.0 {
                    //         let tm = traffic_models.get(edge_id as usize).unwrap();
                    //         tm.debug();
                    //         panic!("Estimated travel time for edge {} is negative: {}", edge_ids[edge_id as usize], estimated_tt);
                    //     }
                    // }

                    graph.set_weight_for_edge_at_time(edge_id, Timestamp::new(interval_begin), FlWeight::new(estimated_tt));
                    edge.overlap_traveltime = Some(estimated_tt);
                });
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

        fn set_weight_for_edge_at_time(&mut self, _edge_id: u32, _at: Timestamp, _new_weight: FlWeight) {
            // No-op for mock
        }

        fn ipps(&self) -> &Vec<TTFPoint> {
            panic!();
        }
    }

    #[test]
    fn test_same_paths_zero_deltas() {
        // Test case 1: old_paths = new_paths -> all sampled_seconds should remain unchanged
        let mut mock_graph = MockTravelTimeGraph::new(3);
        mock_graph.set_travel_time(0, FlWeight::new(5.0));
        mock_graph.set_travel_time(1, FlWeight::new(3.0));
        mock_graph.set_travel_time(2, FlWeight::new(7.0));

        let old_paths_owned = vec![vec![0], vec![1], vec![2]];
        let old_paths: Vec<&Vec<u32>> = old_paths_owned.iter().collect();
        let new_paths = vec![vec![0], vec![1], vec![2]]; // Same as old_paths
        let departures = vec![Timestamp::new(0.0), Timestamp::new(10.0), Timestamp::new(20.0)];
        let free_flow_tts = vec![1.0, 1.0, 1.0];
        let edge_lanes = vec![1, 1, 1];

        // Create edges for each interval with initial sampled_seconds values
        use conversion::sumo::meandata::Edge;
        let edges0 = vec![Edge {
            id: "edge0".to_string(),
            sampled_seconds: Some(10.0),
            ..Default::default()
        }];
        let edges1 = vec![Edge {
            id: "edge1".to_string(),
            sampled_seconds: Some(10.0),
            ..Default::default()
        }];
        let edges2 = vec![Edge {
            id: "edge2".to_string(),
            sampled_seconds: Some(10.0),
            ..Default::default()
        }];

        let mut intervals = vec![
            Interval::create("0".to_string(), 0.0, 10.0, edges0),
            Interval::create("1".to_string(), 10.0, 20.0, edges1),
            Interval::create("2".to_string(), 20.0, 30.0, edges2),
        ];
        let edge_ids = vec!["edge0".to_string(), "edge1".to_string(), "edge2".to_string()];
        let edge_lengths = vec![100.0, 150.0, 200.0];
        let traffic_models: Vec<Box<dyn TrafficModel>> = vec![];

        // Record initial sampled_seconds values
        let initial_sampled0 = intervals[0].get_edge_mut("edge0").unwrap().sampled_seconds.unwrap_or(0.0);
        let initial_sampled1 = intervals[1].get_edge_mut("edge1").unwrap().sampled_seconds.unwrap_or(0.0);
        let initial_sampled2 = intervals[2].get_edge_mut("edge2").unwrap().sampled_seconds.unwrap_or(0.0);

        adjust_weights_in_graph_by_following_paths(
            &mut mock_graph,
            &old_paths,
            &new_paths,
            &departures,
            &mut intervals,
            &edge_ids,
            &edge_lengths,
            &free_flow_tts,
            &traffic_models,
            &edge_lanes,
        );

        // All sampled_seconds should be unchanged since we subtract and add the same values
        let final_sampled0 = intervals[0].get_edge_mut("edge0").unwrap().sampled_seconds.unwrap_or(0.0);
        let final_sampled1 = intervals[1].get_edge_mut("edge1").unwrap().sampled_seconds.unwrap_or(0.0);
        let final_sampled2 = intervals[2].get_edge_mut("edge2").unwrap().sampled_seconds.unwrap_or(0.0);

        assert!(
            (final_sampled0 - initial_sampled0).abs() < 1e-10,
            "Edge 0 sampled_seconds should be unchanged: {} -> {}",
            initial_sampled0,
            final_sampled0
        );
        assert!(
            (final_sampled1 - initial_sampled1).abs() < 1e-10,
            "Edge 1 sampled_seconds should be unchanged: {} -> {}",
            initial_sampled1,
            final_sampled1
        );
        assert!(
            (final_sampled2 - initial_sampled2).abs() < 1e-10,
            "Edge 2 sampled_seconds should be unchanged: {} -> {}",
            initial_sampled2,
            final_sampled2
        );
    }

    #[test]
    fn test_different_paths_nonzero_deltas() {
        // Test case 2: old_paths != new_paths -> sampled_seconds changes accordingly
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
                sampled_seconds: Some(5.0), // Pre-populate with value equal to travel time
                lane_density: None,
                ..Default::default()
            },
            Edge {
                id: "edge1".to_string(),
                traveltime: None,
                density: None,
                speed: None,
                sampled_seconds: Some(0.0),
                lane_density: None,
                ..Default::default()
            },
            Edge {
                id: "edge2".to_string(),
                traveltime: None,
                density: None,
                speed: None,
                sampled_seconds: Some(0.0),
                lane_density: None,
                ..Default::default()
            },
        ];
        let free_flow_tts = vec![1.0, 1.0, 1.0];
        let mut intervals = vec![Interval::create("0".to_string(), 0.0, 10.0, edges)];
        let edge_ids = vec!["edge0".to_string(), "edge1".to_string(), "edge2".to_string()];
        let edge_lengths = vec![100.0, 150.0, 200.0];
        let edge_lanes = vec![1, 1, 1];
        let traffic_models: Vec<Box<dyn TrafficModel>> = vec![];

        adjust_weights_in_graph_by_following_paths(
            &mut mock_graph,
            &old_paths,
            &new_paths,
            &departures,
            &mut intervals,
            &edge_ids,
            &edge_lengths,
            &free_flow_tts,
            &traffic_models,
            &edge_lanes,
        );

        // Edge 0 should have decreased sampled_seconds (old path removed)
        let edge0_sampled = intervals[0].get_edge("edge0").unwrap().sampled_seconds.unwrap_or(0.0);
        assert!(edge0_sampled < 5.0, "Edge 0 should have decreased sampled_seconds, got {}", edge0_sampled);

        // Edge 1 should have increased sampled_seconds (new path added)
        let edge1_sampled = intervals[0].get_edge("edge1").unwrap().sampled_seconds.unwrap_or(0.0);
        assert!(edge1_sampled > 0.0, "Edge 1 should have positive sampled_seconds, got {}", edge1_sampled);

        // Edge 2 should have zero sampled_seconds (not used)
        let edge2_sampled = intervals[0].get_edge("edge2").unwrap().sampled_seconds.unwrap_or(0.0);
        assert!(edge2_sampled.abs() < 1e-10, "Edge 2 should have zero sampled_seconds, got {}", edge2_sampled);
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
        let traffic_models: Vec<Box<dyn TrafficModel>> = vec![];

        // This should panic due to the assertion
        adjust_weights_in_graph_by_following_paths(
            &mut mock_graph,
            &old_paths,
            &new_paths,
            &departures,
            &mut intervals,
            &edge_ids,
            &edge_lengths,
            &free_flow_tts,
            &traffic_models,
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
            lane_density: None,
            ..Default::default()
        }];
        let edges1 = vec![Edge {
            id: "edge0".to_string(),
            traveltime: None,
            density: None,
            speed: None,
            sampled_seconds: Some(0.0),
            lane_density: None,
            ..Default::default()
        }];
        let edges2 = vec![Edge {
            id: "edge0".to_string(),
            traveltime: None,
            density: None,
            speed: None,
            sampled_seconds: Some(0.0),
            lane_density: None,
            ..Default::default()
        }];

        let mut intervals = vec![
            Interval::create("0".to_string(), 0.0, 10.0, edges0),
            Interval::create("1".to_string(), 10.0, 20.0, edges1),
            Interval::create("2".to_string(), 20.0, 30.0, edges2),
        ];
        let edge_ids = vec!["edge0".to_string()];
        let edge_lengths = vec![100.0];
        let free_flow_tts = vec![1.0];
        let edge_lanes = vec![1];
        let traffic_models: Vec<Box<dyn TrafficModel>> = vec![];

        adjust_weights_in_graph_by_following_paths(
            &mut mock_graph,
            &old_paths,
            &new_paths,
            &departures,
            &mut intervals,
            &edge_ids,
            &edge_lengths,
            &free_flow_tts,
            &traffic_models,
            &edge_lanes,
        );

        // Edge 0 should have positive sampled_seconds in first two periods due to overlap
        let sampled0 = intervals[0].get_edge("edge0").unwrap().sampled_seconds.unwrap_or(0.0);
        let sampled1 = intervals[1].get_edge("edge0").unwrap().sampled_seconds.unwrap_or(0.0);
        let sampled2 = intervals[2].get_edge("edge0").map(|e| e.sampled_seconds.unwrap_or(0.0)).unwrap_or(0.0);

        assert!(sampled0 > 0.0, "Period 0 should have positive sampled_seconds, got {}", sampled0);
        assert!(sampled1 > 0.0, "Period 1 should have positive sampled_seconds, got {}", sampled1);
        assert!(sampled2.abs() < 1e-10, "Period 2 should have zero sampled_seconds, got {}", sampled2);

        // The total sampled_seconds should equal the travel time
        let total_sampled = sampled0 + sampled1 + sampled2;
        assert!(
            (total_sampled - 15.0).abs() < 1e-10,
            "Total sampled_seconds should equal travel time, got {}",
            total_sampled
        );
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
            lane_density: None,
            ..Default::default()
        }];
        let edges1 = vec![Edge {
            id: "edge0".to_string(),
            traveltime: None,
            density: None,
            speed: None,
            sampled_seconds: Some(0.0),
            lane_density: None,
            ..Default::default()
        }];
        let edges2 = vec![Edge {
            id: "edge0".to_string(),
            traveltime: None,
            density: None,
            speed: None,
            sampled_seconds: Some(0.0),
            lane_density: None,
            ..Default::default()
        }];

        let free_flow_tts = vec![1.0];
        let mut intervals = vec![
            Interval::create("0".to_string(), 0.0, 10.0, edges0),
            Interval::create("1".to_string(), 10.0, 20.0, edges1),
            Interval::create("2".to_string(), 20.0, 30.0, edges2),
        ];
        let edge_ids = vec!["edge0".to_string()];
        let edge_lengths = vec![100.0];
        let edge_lanes = vec![1];
        let traffic_models: Vec<Box<dyn TrafficModel>> = vec![];

        adjust_weights_in_graph_by_following_paths(
            &mut mock_graph,
            &old_paths,
            &new_paths,
            &departures,
            &mut intervals,
            &edge_ids,
            &edge_lengths,
            &free_flow_tts,
            &traffic_models,
            &edge_lanes,
        );

        // Expected distribution:
        // Period 0 (0-10): overlap from t=7 to t=10 = 3 seconds
        // Period 1 (10-20): overlap from t=10 to t=13 = 3 seconds
        // Period 2 (20-30): no overlap = 0 seconds

        let sampled0 = intervals[0].get_edge("edge0").unwrap().sampled_seconds.unwrap_or(0.0);
        let sampled1 = intervals[1].get_edge("edge0").unwrap().sampled_seconds.unwrap_or(0.0);
        let sampled2 = intervals[2].get_edge("edge0").map(|e| e.sampled_seconds.unwrap_or(0.0)).unwrap_or(0.0);

        assert!((sampled0 - 3.0).abs() < 1e-10, "Period 0 should have 3.0 sampled_seconds, got {}", sampled0);
        assert!((sampled1 - 3.0).abs() < 1e-10, "Period 1 should have 3.0 sampled_seconds, got {}", sampled1);
        assert!(sampled2.abs() < 1e-10, "Period 2 should have zero sampled_seconds, got {}", sampled2);

        // Total should equal travel time
        let total_sampled = sampled0 + sampled1 + sampled2;
        assert!(
            (total_sampled - 6.0).abs() < 1e-10,
            "Total sampled_seconds should equal travel time of 6.0, got {}",
            total_sampled
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
            lane_density: None,
            ..Default::default()
        }];
        let edges1 = vec![Edge {
            id: "edge0".to_string(),
            traveltime: None,
            density: None,
            speed: None,
            sampled_seconds: Some(0.0),
            lane_density: None,
            ..Default::default()
        }];
        let edges2 = vec![Edge {
            id: "edge0".to_string(),
            traveltime: None,
            density: None,
            speed: None,
            sampled_seconds: Some(0.0),
            lane_density: None,
            ..Default::default()
        }];
        let edges3 = vec![Edge {
            id: "edge0".to_string(),
            traveltime: None,
            density: None,
            speed: None,
            sampled_seconds: Some(0.0),
            lane_density: None,
            ..Default::default()
        }];

        let free_flow_tts = vec![1.0];
        let mut intervals = vec![
            Interval::create("0".to_string(), 0.0, 10.0, edges0),
            Interval::create("1".to_string(), 10.0, 15.0, edges1),
            Interval::create("2".to_string(), 15.0, 25.0, edges2),
            Interval::create("3".to_string(), 25.0, 35.0, edges3),
        ];
        let edge_ids = vec!["edge0".to_string()];
        let edge_lengths = vec![100.0];
        let edge_lanes = vec![1];
        let traffic_models: Vec<Box<dyn TrafficModel>> = vec![];

        adjust_weights_in_graph_by_following_paths(
            &mut mock_graph,
            &old_paths,
            &new_paths,
            &departures,
            &mut intervals,
            &edge_ids,
            &edge_lengths,
            &free_flow_tts,
            &traffic_models,
            &edge_lanes,
        );

        // Expected distribution:
        // Period 0 (0-10): overlap from t=4 to t=10 = 6 seconds
        // Period 1 (10-15): overlap from t=10 to t=15 = 5 seconds
        // Period 2 (15-25): overlap from t=15 to t=22 = 7 seconds
        // Period 3 (25-35): no overlap = 0 seconds

        let sampled0 = intervals[0].get_edge("edge0").unwrap().sampled_seconds.unwrap_or(0.0);
        let sampled1 = intervals[1].get_edge("edge0").unwrap().sampled_seconds.unwrap_or(0.0);
        let sampled2 = intervals[2].get_edge("edge0").unwrap().sampled_seconds.unwrap_or(0.0);
        let sampled3 = intervals[3].get_edge("edge0").map(|e| e.sampled_seconds.unwrap_or(0.0)).unwrap_or(0.0);

        assert!((sampled0 - 6.0).abs() < 1e-10, "Period 0 should have 6.0 sampled_seconds, got {}", sampled0);
        assert!((sampled1 - 5.0).abs() < 1e-10, "Period 1 should have 5.0 sampled_seconds, got {}", sampled1);
        assert!((sampled2 - 7.0).abs() < 1e-10, "Period 2 should have 7.0 sampled_seconds, got {}", sampled2);
        assert!(sampled3.abs() < 1e-10, "Period 3 should have zero sampled_seconds, got {}", sampled3);

        // Total should equal travel time
        let total_sampled = sampled0 + sampled1 + sampled2 + sampled3;
        assert!(
            (total_sampled - 18.0).abs() < 1e-10,
            "Total sampled_seconds should equal travel time of 18.0, got {}",
            total_sampled
        );
    }
}
