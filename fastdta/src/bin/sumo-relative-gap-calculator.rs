use clap::Parser;
use conversion::{
    sumo::{
        routes::Vehicle, routes_reader::SumoRoutesReader, sumo_find_file::get_routes_file_name_in_iteration, sumo_to_new_graph_weights::extract_travel_times_from_iteration_directory, sumo_to_td_graph_converter::convert_sumo_to_routing_kit_and_queries, FileReader, SumoTravelTime
    }, FILE_EDGE_INDICES_TO_ID, FILE_QUERY_IDS
};
use std::{collections::HashMap, env, fs::OpenOptions, path::Path};
use std::{fs::remove_dir_all, io::Write};

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

    convert_sumo_to_routing_kit_and_queries(&network_dir, &network_prefix, &trips_file, &temp_cch_dir).unwrap();

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

    let mut iteration: u32 = if let Some(it) = args.iteration { it } else { 0 };
    let mut dta_iteration_dir = dta_dir.join(format!("{:0>3}", iteration));

    while dta_iteration_dir.exists() {
        extract_travel_times_from_iteration_directory(&dta_iteration_dir, &temp_cch_dir, &edge_ids);
        let graph = TDGraph::reconstruct_from(&temp_cch_dir).unwrap();
        let cch = get_cch(&temp_cch_dir, &graph);
        let customized_graph = customize(&cch, &graph);
        let (best_paths, best_travel_times, _) = get_paths_with_cch(&cch, &customized_graph, &temp_cch_dir, &graph);

        let routes_path = dta_iteration_dir.join(get_routes_file_name_in_iteration(&trips_file, iteration));
        dbg!("Reading routes from file {}", routes_path.display());
        let routes_document_root = SumoRoutesReader::read(&routes_path).unwrap();
        let vehicle_id_to_vehicle: HashMap<&String, &Vehicle> = routes_document_root.vehicles.iter().map(|v| (&v.id, v)).collect();

        let experienced_tt: Vec<SumoTravelTime> = query_ids
            .par_iter()
            .enumerate()
            .map(|(i, id)| {
                if let Some(v) = vehicle_id_to_vehicle.get(id) {
                    let experienced_path: Vec<u32> = if let Some(route) = &v.route {
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

                    let experienced_time = graph.get_travel_time_along_path(Timestamp::new(v.depart), &experienced_path);
                    let experienced_time_f64: f64 = <FlWeight as Into<f64>>::into(experienced_time);
                    let best_time_f64: f64 = <FlWeight as Into<f64>>::into(best_travel_times[i]);

                    
                    if (experienced_time_f64 - best_time_f64) < -EPSILON_TRAVEL_TIME {
                        // print a debug message containing vehicle id, experienced time, best time, and both paths + departure time
                        eprintln!(
                            "Warning: Experienced travel time for vehicle id {} is less than best travel time: \n{} < {}.\nExperienced path: {:?}, \nbest path:        {:?},\ndeparture time: {}",
                            id,
                            experienced_time_f64,
                            best_time_f64,
                            get_path_ids_from_indices(&edge_ids, &experienced_path),
                            get_path_ids_from_indices(&edge_ids, &best_paths[i]),
                            v.depart
                        );
                    }

                    experienced_time.into()
                } else {
                    // negative number will be ignored in the relative gap calculation.
                    panic!("No route found for query id {}", id);
                }
            })
            .collect();

        let best_tt: Vec<SumoTravelTime> = best_travel_times.par_iter().map(|&tt| tt.into()).collect();

        print_network_travel_time(&experienced_tt);
        print_highest_differences(&best_tt, &experienced_tt, &best_paths, &routes_document_root.vehicles, &query_ids,&edge_ids);

        let rel_gap = get_relative_gap(&best_tt, &experienced_tt);

        rel_gaps.push(rel_gap);

        iteration += 1;
        dta_iteration_dir = dta_dir.join(format!("{:0>3}", iteration));
    }

    // append each gap to a file "rel_gaps.txt" in the dta_dir
    let mut file = OpenOptions::new().create(true).append(true).open(Path::new(&args.output_file)).unwrap();
    for gap in rel_gaps {
        // write at the end of the file:
        writeln!(file, "{:.6}", gap).unwrap();
    }

    // remove the temporary CCH directory
    remove_dir_all(&temp_cch_dir).unwrap();
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

    /// the output file to write the relative gaps to
    #[arg(long = "output-file", default_value = "rel_gaps.txt")]
    pub output_file: String,

    /// the iteration to read from the dta directory (optional: defaults to read all iterations)
    /// If not specified, the whole directory will be read
    /// If specified, only the files for that iteration will be read
    #[arg(long = "iteration")]
    pub iteration: Option<u32>,
}

/// prints the experienced total network travel time
fn print_network_travel_time(
    experienced_tts: &Vec<SumoTravelTime>
) {
    let total_experienced_tt: f64 = experienced_tts
        .par_iter()
        .map(|&tt| <f64 as From<SumoTravelTime>>::from(tt))
        .sum();

    println!("Total experienced travel time: {:.6} seconds", total_experienced_tt);
}

fn print_highest_differences(
    best_tts: &Vec<SumoTravelTime>,
    experienced_tts: &Vec<SumoTravelTime>,
    best_paths: &Vec<Vec<u32>>, 
    vehicles: &Vec<Vehicle>,
    query_ids: &Vec<String>,
    edge_ids: &Vec<String>
) {
    let mut differences: Vec<(usize, f64)> = best_tts
        .par_iter()
        .enumerate()
        .map(|(i, &best_tt)| {
            let experienced_tt_f64: f64 = <f64 as From<SumoTravelTime>>::from(experienced_tts[i]);
            let best_tt_f64: f64 = <f64 as From<SumoTravelTime>>::from(best_tt);
            let diff = experienced_tt_f64 - best_tt_f64;
            (i, diff)
        })
        .collect();

    differences.par_sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("Top 20 highest differences between experienced and best travel times:");
    for i in 0..20.min(differences.len()) {
        let (index, diff) = differences[i];
        if diff > EPSILON_TRAVEL_TIME {
            println!(
                "Query ID: {}, Difference: {:.6}, \nBest TT: {:.6}, \nPath: {}\n Experienced TT: {:.6}, \nPath: {}",
                query_ids[index],
                diff,
                <f64 as From<SumoTravelTime>>::from(best_tts[index]),
                get_path_ids_from_indices(edge_ids, &best_paths[index]).join(" "),
                <f64 as From<SumoTravelTime>>::from(experienced_tts[index]),
                vehicles[index].route.as_ref().map_or(String::from("No route"), |r| r.edges.clone())
            );
        }
    }
}