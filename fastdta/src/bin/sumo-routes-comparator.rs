use clap::Parser;
use conversion::{
    FILE_EDGE_INDICES_TO_ID, FILE_QUERY_IDS,
    sumo::{
        FileReader, SumoTravelTime, routes::Vehicle, routes_reader::SumoRoutesReader, sumo_to_new_graph_weights::extract_travel_times_from_iteration_directory,
        sumo_to_td_graph_converter::convert_sumo_to_routing_kit_and_queries,
    },
};
use std::fs::remove_dir_all;
use std::{collections::HashMap, env, path::Path};

use fastdta::{
    customize::customize,
    preprocess::{get_cch, preprocess, run_inertial_flow_cutter},
    query::get_paths_with_cch,
    relative_gap::{EPSILON_TRAVEL_TIME, get_relative_gap},
};
use rayon::prelude::*;
use rust_road_router::{
    datastr::graph::floating_time_dependent::{FlWeight, Timestamp},
    io::Reconstruct,
};
use rust_road_router::{datastr::graph::floating_time_dependent::TDGraph, io::read_strings_from_file};

/// arguments:
/// * trip file: contains sumo trips
/// * network directory: contains the sumo network (nodes, edges, connections)
/// * dta directory: contains several iteration directories (000, 001, ...) each containing:
///     * dump file: contains travel times for each edge
///     * routes file: contains the routes of the vehicles
///     * alt routes file: contains alternative routes of the vehicles
///
/// This programm compares the shortest routes of the vehicles in the routes file
/// and calculates the shortest routes using CATCHUP with the network and the dumped travel times.
/// If the shortest route differs (i.e. the edges and the travel times are different), a message is printed to the console.

fn main() {
    let args = Args::parse();

    let network_dir = Path::new(&args.net_dir);
    let network_prefix = args.net_prefix;
    let trips_file = Path::new(&args.trips_file);
    // in the dta_dir, there should be several directories in the format "iteration%3i"
    // each subdirectory corresponds to a different iteration of the DTA
    // in each subdirectory, there should be a file "tripinfo.xml" which contains the SUMO trip information and a dump file containing
    // travel times of each edge
    let dta_dir = Path::new(&args.dta_dir);
    let temp_dir_name = "tmp";
    let temp_cch_dir = dta_dir.join(temp_dir_name);

    convert_sumo_to_routing_kit_and_queries(&network_dir, &network_prefix, &trips_file, &temp_cch_dir, None, None, None).unwrap();

    // create a subprocess which runs the bash script: "flow_cutter_cch_cut_order.sh <output_dir>" to create node rankings for the TD-CCH
    run_inertial_flow_cutter(&temp_cch_dir, 42, std::thread::available_parallelism().unwrap().get() as i32).unwrap();

    // run catchup preprocessing
    preprocess(&temp_cch_dir).unwrap();

    // read the "query_id" file from the tmp dir:
    let query_ids: Vec<String> = read_strings_from_file(&temp_cch_dir.join(FILE_QUERY_IDS)).unwrap();

    let edge_ids: Vec<String> = read_strings_from_file(&temp_cch_dir.join(FILE_EDGE_INDICES_TO_ID)).unwrap_or_else(|_| {
        panic!(
            "Failed to read edge indices from file {} in directory {}",
            FILE_EDGE_INDICES_TO_ID,
            temp_cch_dir.display()
        )
    });

    let edge_id_to_index: HashMap<&String, usize> = edge_ids.iter().enumerate().map(|(i, id)| (id, i)).collect();

    let mut rel_gaps: Vec<f64> = Vec::new();

    let mut stop_after_first_iteration = false;

    let mut iteration: u32 = if let Some(it) = args.iteration {
        if it == 0 {
            panic!("Iteration must be greater than 0 to use previous iteration's travel times.");
        }
        stop_after_first_iteration = true;
        it
    } else {
        1 // start from 1 if not specified
    };
    let mut dta_iteration_dir = dta_dir.join(format!("{:0>3}", iteration));

    while dta_iteration_dir.exists() {
        // Use travel times from previous iteration
        let prev_iteration = iteration - 1;
        let prev_dta_iteration_dir = dta_dir.join(format!("{:0>3}", prev_iteration));
        if !prev_dta_iteration_dir.exists() {
            panic!("Previous iteration directory {} does not exist.", prev_dta_iteration_dir.display());
        }
        extract_travel_times_from_iteration_directory(&prev_dta_iteration_dir, &temp_cch_dir, &edge_ids);
        let graph = TDGraph::reconstruct_from(&temp_cch_dir).unwrap();
        let cch = get_cch(&temp_cch_dir, &graph);
        let customized_graph = customize(&cch, &graph);
        let (paths, travel_times, _) = get_paths_with_cch(&cch, &customized_graph, &temp_cch_dir, &graph);

        let routes_path = dta_iteration_dir.join(get_routes_file_name_in_iteration(&trips_file, iteration));
        println!("Reading routes from file {}", routes_path.display());
        let routes_document_root = SumoRoutesReader::read(&routes_path).unwrap();
        let route_id_to_route: HashMap<&String, &Vehicle> = routes_document_root.vehicles.iter().map(|v| (&v.id, v)).collect();

        let experienced_tt: Vec<SumoTravelTime> = query_ids
            .par_iter()
            .enumerate()
            .map(|(i, id)| {
                if let Some(v) = route_id_to_route.get(id) {
                    let path: Vec<u32> = if let Some(route) = &v.route {
                        route
                            .edges
                            .split_ascii_whitespace()
                            .map(|edge_id| {
                                if let Some(&index) = edge_id_to_index.get(&edge_id.to_string()) {
                                    index as u32
                                } else {
                                    panic!("Edge id {} not found in edge_id_to_index map", edge_id);
                                }
                            })
                            .collect()
                    } else {
                        panic!("No route found for vehicle id {}", id);
                    };

                    let experienced_time = graph.get_travel_time_along_path(Timestamp::new(v.depart), &path);
                    let best_path = &paths[i];
                    let best_time = travel_times[i];

                    // Compare edge sequences and travel times
                    let path_ids = get_path_ids_from_indices(&edge_ids, &path);
                    let best_path_ids = get_path_ids_from_indices(&edge_ids, best_path);
                    let experienced_time_f64: f64 = <FlWeight as Into<f64>>::into(experienced_time);
                    let best_time_f64: f64 = <FlWeight as Into<f64>>::into(best_time);

                    // if path_ids != best_path_ids && (experienced_time_f64 - best_time_f64).abs() > EPSILON_TRAVEL_TIME {
                    //     println!(
                    //         "Route mismatch for vehicle id {}:\n  SUMO route:      {:?}\n  SUMO travel time: {:.3}\n  CATCHUP route:   {:?}\n  CATCHUP travel time: {:.3}\n  departure time: {}",
                    //         id,
                    //         path_ids,
                    //         experienced_time_f64,
                    //         best_path_ids,
                    //         best_time_f64,
                    //         v.depart
                    //     );
                    // }

                    // if experienced time < best time, print a warning that the best time is not optimal for the current id 
                    // also print the sumo route and the catchup route
                    if experienced_time_f64 + EPSILON_TRAVEL_TIME < best_time_f64 {
                        println!(
                            "Warning: Experienced travel time is less than best travel time for vehicle id {}:\n  SUMO route:      {:?}\n  SUMO travel time: {:.3}\nCATCHUP route:   {:?}\n  CATCHUP travel time: {:.3}\n  departure time: {}",
                            id,
                            path_ids,
                            experienced_time_f64,
                            best_path_ids,
                            best_time_f64,
                            v.depart
                        );
                    }
                    

                    experienced_time.into()
                } else {
                    // negative number will be ignored in the relative gap calculation.
                    -1.0
                }
            })
            .collect();

        let best_tt: Vec<SumoTravelTime> = travel_times.par_iter().map(|&tt| tt.into()).collect();

        let rel_gap = get_relative_gap(&best_tt, &experienced_tt);

        rel_gaps.push(rel_gap);

        iteration += 1;
        dta_iteration_dir = dta_dir.join(format!("{:0>3}", iteration));

        if stop_after_first_iteration {
            break;
        }
    }

    // remove the temporary CCH directory
    remove_dir_all(&temp_cch_dir).unwrap();
}

/// trips_file has the following format: "<name>[.trips].xml"
/// the routes file in iteration <iteration> should have the following name:
/// "<name>_<iteration>.rou.xml"
pub fn get_routes_file_name_in_iteration(trips_file: &Path, iteration: u32) -> String {
    let mut file_stem = trips_file.file_stem().unwrap().to_str().unwrap();
    file_stem = if file_stem.ends_with(".trips") {
        &file_stem[..file_stem.len() - 6]
    } else {
        file_stem
    };
    let routes_file_name = format!("{}_{}.rou.xml", file_stem, format!("{:0>3}", iteration));
    routes_file_name
}

pub fn get_path_ids_from_indices(edge_ids: &Vec<String>, indices: &Vec<u32>) -> Vec<String> {
    indices.iter().map(|&i| edge_ids[i as usize].clone()).collect()
}

/// Command-line arguments for counting connections and whether they are complete or not
#[derive(Parser, Debug)]
#[command(version, about = "Sumo Relative Gap Calculator options", long_about = None)]
pub struct Args {
    /// the directory containing the input files
    #[arg(long = "net-dir", default_value_t = String::from(env::current_dir().unwrap().to_str().unwrap()))]
    pub net_dir: String,

    /// the files `<net-prefix>.nod.xml`, `<net-prefix>.edg.xml` will be read as input
    #[arg(long = "net-prefix", default_value = "")]
    pub net_prefix: String,

    /// the trips file to read
    #[arg(long = "trips-file")]
    pub trips_file: String,

    /// the root directory in which dta was conducted (optional: defaults to current directory)
    #[arg(long = "dta-dir", default_value_t = String::from(env::current_dir().unwrap().to_str().unwrap()))]
    pub dta_dir: String,

    /// the iteration to read from the dta directory (optional: defaults to read all iterations)
    /// If not specified, the whole directory will be read
    /// If specified, only the files for that iteration will be read
    #[arg(long = "iteration")]
    pub iteration: Option<u32>,
}
