use rust_road_router::datastr::graph::{time_dependent::*, *};

pub mod here;
pub mod sumo;

pub const FILE_LATITUDE: &str = "latitude";
pub const FILE_LONGITUDE: &str = "longitude";
pub const FILE_CCH_PERM: &str = "cch_perm";
pub const FILE_CCH_SEPARATORS: &str = "cch_separators";
pub const FILE_CCH_NODE_ORDER: &str = "cch_node_order";
pub const DIR_CCH: &str = "cch";
pub const DIR_DTA: &str = "dta";
pub const DIR_CUSTOMIZED: &str = "customized";
pub const FILE_FIRST_OUT: &str = "first_out";
pub const FILE_HEAD: &str = "head";
pub const FILE_FIRST_IPP_OF_ARC: &str = "first_ipp_of_arc";
pub const FILE_IPP_DEPARTURE_TIME: &str = "ipp_departure_time";
pub const FILE_IPP_TRAVEL_TIME: &str = "ipp_travel_time";
pub const FILE_QUERY_IDS: &str = "query_ids";
pub const FILE_QUERY_ORIGINAL_FROM_EDGES: &str = "query_original_from_edges";
pub const FILE_QUERY_ORIGINAL_TO_EDGES: &str = "query_original_to_edges";
pub const FILE_QUERIES_FROM: &str = "queries_from";
pub const FILE_QUERIES_TO: &str = "queries_to";
pub const FILE_QUERIES_DEPARTURE: &str = "queries_departure";
/// contains all edges of all alternative paths of queries used during DTA, where edges are encoded as u32 indices
pub const FILE_DTA_QUERIES_EDGE_IDS: &str = "edge_ids";
/// contains the index of the first alternative to a set of alternative paths.
/// Query with index i has its first alternative path starting at edge_ids[first_edge_of_alternative[first_alternative_of_query[i]]]
/// Number of alternatives for query i is given by first_alternative_of_query[i + 1] - first_alternative_of_query[i]
pub const FILE_DTA_QUERIES_FIRST_ALTERNATIVE_PATH: &str = "first_alternative_of_query";
/// Contains the index of the first edge of the alternative path.
/// length of a path is given by first_edge_of_alternative[i + 1] - first_edge_of_alternative[i]
pub const FILE_DTA_QUERIES_FIRST_EDGE_OF_ALTERNATIVE: &str = "first_edge_of_alternative";
/// contains the stochastic cost of an alternative path
/// the cost of the i-th alternative path of query j is given by
/// alternative_costs[first_alternative_of_query[j] + i]
pub const FILE_DTA_QUERIES_ALTERNATIVE_COST: &str = "alternative_cost";
/// contains information which alternative path was chosen for each query
/// if i-th alternative path of query j was chosen, then
/// alternative_choice[j] = i
pub const FILE_DTA_QUERIES_ALTERNATIVE_CHOICE: &str = "alternative_choice";
/// contains the probability of choosing an alternative path
/// the probability of the i-th alternative path of query j is given by
/// alternative_probabilities[first_alternative_of_query[j] + i]
pub const FILE_DTA_QUERIES_ALTNERNATIVE_PROBABILITIES: &str = "alternative_probabilities";

/// each line i contains the original id of the edge i
pub const FILE_EDGE_INDICES_TO_ID: &str = "edge_indices_to_id";

// contains the default travel times calculated during the preprocessing step
// the travel times are encoded as SerializedTravelTime in milliseconds
pub const FILE_EDGE_DEFAULT_TRAVEL_TIMES: &str = "edge_default_travel_times";

// contains the capacities of the edges calculated during the preprocessing step
pub const FILE_EDGE_CAPACITIES: &str = "edge_capacities";

// contains the lengths of the edges
pub const FILE_EDGE_LENGTHS: &str = "edge_lengths";

// contains the number of lanes of the edges
pub const FILE_EDGE_LANES: &str = "edge_lanes";

// contains the speed in m/s of the edges
pub const FILE_EDGE_SPEEDS: &str = "edge_speed";

/// contains the traffic model parameters for each edge
/// the parameters are stored in the same order for each edge as f64 values
pub const FILE_EDGE_TRAFFIC_MODEL_PARAMS: &str = "edge_traffic_model_params";

/// edge with index `i` has its model parameters stored
/// starting from index `first_model_param_of_edge[i]` in `model_params`
pub const FILE_EDGE_FIRST_MODEL_PARAM: &str = "first_model_param_of_edge";

/// contains lane density observations for edges (vehicles per km) as f64 values
pub const FILE_EDGE_DENSITY_OBSERVATIONS: &str = "edge_density_observations";

/// edge with index `i` has its density observations stored
/// starting from index `first_density_observation_of_edge[i]` in `density_observations`
pub const FILE_EDGE_FIRST_DENSITY_OBSERVATION: &str = "first_density_observation_of_edge";

/// contains speed observations for edges (m/s) as f64 values
pub const FILE_EDGE_SPEED_OBSERVATIONS: &str = "edge_speed_observations";

/// edge with index `i` has its speed observations stored
/// starting from index `first_speed_observation_of_edge[i]` in `speed_observations`
pub const FILE_EDGE_FIRST_SPEED_OBSERVATION: &str = "first_speed_observation_of_edge";

pub const FILE_RELATIVE_GAPS: &str = "rel_gaps.txt";

/// in milliseconds
pub type SerializedTravelTime = u32;
/// in milliseconds
pub type SerializedTimestamp = u32;
/// in meters
pub type SerializedPosition = f32;

/// lowest possible travel time for an edge in seconds
pub const MIN_EDGE_WEIGHT: f64 = 1.0;

/// factor to multiply free-flow speeds with to get global free-flow speeds
/// [1] uses a factor of below 1.0 which accounts for turning delays at intersections or red lights in urbad areas.
/// However, we dont consider traffic lights or turning delays in our model yet, so we keep the factor at 1.0 for now.
///
/// [1]: https://www.sciencedirect.com/science/article/pii/S1877050924013164
pub const GLOBAL_FREE_FLOW_SPEED_FACTOR: f64 = 1.0;

pub const MAX_EDGE_WEIGHT: f64 = 86_400.0; // 24 hours in seconds
pub const SUMO_MAX_TRAVEL_TIME: f64 = 100_000.0; // 100k seconds
pub const SUMO_DEFAULT_SPEED: f64 = 13.9; // 50 km/h in m/s

pub fn speed_profile_to_tt_profile(speeds: &[(Timestamp, u32)], edge_len: u32) -> Vec<(Timestamp, Weight)> {
    let t_wrap = speeds.last().unwrap().0;
    let last_to_exit = speeds.len() - 2;
    let mut speeds = &*speeds; // reborrow for lifetime foo
    let mut extended_speeds = Vec::new();
    assert!(edge_len > 0);
    assert!(speeds.len() > 1);
    let tt_first = tt(speeds[0].1, edge_len);
    let needs_extension = tt_first > speeds[1].0;
    if needs_extension {
        extended_speeds.extend_from_slice(speeds);
    }

    let mut entered = std::collections::VecDeque::new();
    entered.push_back(0);
    let mut next_to_enter = 1;

    while tt_at_exit(&speeds[*entered.front().unwrap()..=*entered.back().unwrap()], edge_len) > speeds[entered.back().unwrap() + 1].0 {
        let to_add = speeds[next_to_enter];
        extended_speeds.push((to_add.0 + t_wrap, to_add.1));
        entered.push_back(next_to_enter);
        next_to_enter += 1;
    }

    if needs_extension {
        let to_add = speeds[next_to_enter];
        extended_speeds.push((to_add.0 + t_wrap, to_add.1));
        speeds = &extended_speeds;
    }

    let mut profile = Vec::new();
    debug_assert!(tt_first > 0);
    profile.push((0, tt_at_exit(&speeds[*entered.front().unwrap()..=*entered.back().unwrap()], edge_len)));

    while *entered.front().unwrap() <= last_to_exit {
        let next_to_exit = entered.pop_front().unwrap();
        let t_exit = speeds[next_to_exit + 1].0;

        if entered.is_empty() {
            let last_tt = profile.last().unwrap().1;
            let t_enter = t_exit - last_tt;
            if profile.last() != Some(&(t_enter, last_tt)) {
                profile
                    .last()
                    .map(|&(t, _)| debug_assert!(t < t_enter, "{:#?}", (&profile, t_exit, t_enter, last_tt)));
                profile.push((t_enter, last_tt));
            }
            entered.push_back(next_to_enter);
            next_to_enter += 1;
        }

        while entered.back().unwrap() + 1 < speeds.len()
            && tt_at_exit(&speeds[*entered.front().unwrap()..=*entered.back().unwrap()], edge_len) + t_exit > speeds[entered.back().unwrap() + 1].0
        {
            let last = profile.last().unwrap();
            let tt_exit = tt_at_exit(&speeds[*entered.front().unwrap()..=*entered.back().unwrap()], edge_len);
            let delta_at = tt_exit + t_exit - (last.0 + last.1);
            let eval_at = speeds[entered.back().unwrap() + 1].0 - (last.0 + last.1);
            let delta_dt = t_exit - last.0;
            let t_enter = Weight::try_from(u64::from(eval_at) * u64::from(delta_dt) / u64::from(delta_at)).unwrap() + last.0;
            let tt_enter = speeds[entered.back().unwrap() + 1].0 - t_enter;

            if let Some(&(t, tt)) = profile.last() {
                if t >= t_enter {
                    assert_eq!(t, t_enter);
                    assert_eq!(tt, tt_enter);
                    profile.pop();
                }
            }
            profile.push((t_enter, tt_enter));
            entered.push_back(next_to_enter);
            next_to_enter += 1;
        }
        profile.last().map(|&(t, _)| debug_assert!(t < t_exit, "{:#?}", (&profile, t_exit, &speeds)));
        profile.push((t_exit, tt_at_exit(&speeds[*entered.front().unwrap()..=*entered.back().unwrap()], edge_len)))
    }

    debug_assert_eq!(
        profile.last().unwrap().0,
        t_wrap,
        "{:#?}",
        (speeds, &profile, &entered, speeds.len(), needs_extension)
    );
    debug_assert_eq!(
        profile[0].1,
        profile.last().unwrap().1,
        "{:#?}",
        (speeds, &profile, &entered, speeds.len(), needs_extension)
    );
    profile
}

fn tt_at_exit(entered_speeds: &[(Timestamp, u32)], len_m: u32) -> Weight {
    match entered_speeds {
        [(_at, speed)] => tt(*speed, len_m),
        [(at, speed), rest @ ..] => {
            let t_cur = rest[0].0 - at;
            t_cur + tt_at_exit(rest, len_m - speed * t_cur / 3600)
        }
        _ => unreachable!(),
    }
}

fn tt(speed_km_h: u32, len_m: u32) -> Weight {
    // if speed_km_h == 0 {
    //     return INFINITY;
    // }
    100 * 36 * len_m / speed_km_h
}
