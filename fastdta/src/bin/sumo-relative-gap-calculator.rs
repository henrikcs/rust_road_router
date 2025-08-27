use clap::Parser;
use conversion::{
    FILE_EDGE_INDICES_TO_ID, FILE_QUERY_IDS,
    sumo::{
        FileReader, SumoTravelTime, sumo_to_new_graph_weights::extract_travel_times_from_previous_iteration,
        sumo_to_td_graph_converter::convert_sumo_to_routing_kit_and_queries, tripinfo::Tripinfo, tripinfo_reader::SumoTripinfoReader,
    },
};
use std::{
    collections::HashMap,
    env,
    fs::{File, OpenOptions},
    path::Path,
};
use std::{fs::remove_dir_all, io::Write};

use fastdta::{
    customize::customize,
    preprocess::{get_cch, preprocess, run_inertial_flow_cutter},
    query::get_paths_with_cch_queries,
    relative_gap::get_relative_gap,
};
use rayon::prelude::*;
use rust_road_router::{algo::catchup::Server, io::Reconstruct};
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

    let mut rel_gaps: Vec<f64> = Vec::new();

    let mut iteration: u32 = if let Some(it) = args.iteration { it } else { 0 };
    let mut dta_iteration_dir = dta_dir.join(format!("{:0>3}", iteration));

    while dta_iteration_dir.exists() {
        let tripinfos_path = dta_iteration_dir.join(format!("tripinfo_{}.xml", format!("{:0>3}", iteration)));
        if !tripinfos_path.exists() {
            panic!(
                "Tripinfo file {} does not exist in directory {}",
                tripinfos_path.display(),
                dta_iteration_dir.display()
            );
        }
        extract_travel_times_from_previous_iteration(&dta_iteration_dir, &temp_cch_dir, &edge_ids);

        let graph = TDGraph::reconstruct_from(&temp_cch_dir).unwrap();

        let cch = get_cch(&temp_cch_dir, &graph);

        let customized_graph = customize(&cch, &graph);

        let (_, travel_times, _) = get_paths_with_cch_queries(&mut Server::new(&cch, &customized_graph), &temp_cch_dir, &graph);

        // the relative gap is calculated during DTA in each iteration
        // in order to calculate the relative gap, we need the shortest paths during each iteration and the corresponding experienced traveltimes from the simulation.
        // the experienced traveltimes are obtained from the SUMO simulation output ("tripinfo.xml")
        // the shortest paths are obtained from calculating the traveltimes of the shortest paths from the previous iteration.
        let tripinfos_document_root = SumoTripinfoReader::read(&tripinfos_path).unwrap();

        // map tripinfos to a map from string to tripinfo, where string ist tripinfo.id
        let tripinfo_map: HashMap<&String, &Tripinfo> = tripinfos_document_root.tripinfos.iter().map(|tripinfo| (&tripinfo.id, tripinfo)).collect();

        let experienced_tt: Vec<SumoTravelTime> = query_ids
            .par_iter()
            .map(|id| {
                let tripinfo = tripinfo_map
                    .get(id)
                    .expect(&format!("Tripinfo with id {} not found in iteration {}", id, iteration));
                tripinfo.duration.into()
            })
            .collect();

        let best_tt: Vec<SumoTravelTime> = travel_times.par_iter().map(|&tt| tt.into()).collect();

        let rel_gap = get_relative_gap(&best_tt, &experienced_tt);

        rel_gaps.push(rel_gap);

        iteration += 1;
        dta_iteration_dir = dta_dir.join(format!("{:0>3}", iteration));
    }

    // append each gap to a file "rel_gaps.txt" in the dta_dir
    let mut file = OpenOptions::new().create(true).append(true).open(dta_dir.join("rel_gaps.txt")).unwrap();
    for gap in rel_gaps {
        // write at the end of the file:
        writeln!(file, "{:.6}", gap).unwrap();
    }

    // remove the temporary CCH directory
    remove_dir_all(&temp_cch_dir).unwrap();
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

    /// the trips file to read inside the input directory
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
