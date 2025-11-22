use std::env;

pub use clap::Parser;

use crate::{choice, traffic_model::TrafficModelType};

/// Command-line arguments for preprocessing
#[derive(Parser, Debug)]
#[command(version, about = "preprocessing CLI options", long_about = None)]
pub struct PreprocesserArgs {
    /// the directory containing the input files
    #[arg(long = "input-dir", default_value_t = String::from(env::current_dir().unwrap().to_str().unwrap()))]
    pub input_dir: String,

    /// the files `<input-prefix>.nod.xml`, `<input-prefix>.edg.xml` will be read as input
    #[arg(long = "input-prefix", default_value = "")]
    pub input_prefix: String,

    /// the trips file to read inside the input directory
    #[arg(long = "trips-file")]
    pub trips_file: String,

    /// the directory to write the output files to (optional, defaults to current directory)
    #[arg(long = "output-dir", default_value_t = String::from(env::current_dir().unwrap().to_str().unwrap()))]
    pub output_dir: String,

    /// the random seed to use for the inertial flow cutter (optional, defaults to 5489)
    #[arg(long = "seed", default_value_t = 5489)]
    pub seed: i32,

    /// the number of threads to use for the routing
    /// (optional, defaults to the number of available threads)
    #[arg(long = "routing-threads", default_value_t = std::thread::available_parallelism().unwrap().get() as i32)]
    pub routing_threads: i32,
}

/// Command-line arguments for routing, derived from duarouter
#[derive(Parser, Debug)]
#[command(version, about = "routing CLI options", long_about = None)]
pub struct RouterArgs {
    #[arg(short = 'd', long = "additional-files")]
    pub additional_files: Option<String>,

    #[arg(long = "aggregate-warnings")]
    pub aggregate_warnings: Option<i32>,

    #[arg(long = "alternatives-output")]
    pub alternatives_output: Option<String>,

    #[arg(long = "arrivallane")]
    pub arrivallane: Option<String>,

    #[arg(long = "arrivalpos")]
    pub arrivalpos: Option<String>,

    #[arg(long = "arrivalspeed")]
    pub arrivalspeed: Option<String>,

    #[arg(long = "astar.all-distances")]
    pub astar_all_distances: Option<String>,

    #[arg(long = "astar.landmark-distances")]
    pub astar_landmark_distances: Option<String>,

    #[arg(long = "astar.save-landmark-distances")]
    pub astar_save_landmark_distances: Option<String>,

    #[arg(long = "begin", short = 'b')]
    pub begin: Option<f64>,

    #[arg(long = "bulk-routing", default_value_t = String::from("False"))]
    pub bulk_routing: String,

    #[arg(long = "configuration-file", short = 'c')]
    pub configuration_file: Option<String>,

    #[arg(long = "defaults-override", default_value_t = String::from("False"))]
    pub defaults_override: String,

    #[arg(long = "departlane")]
    pub departlane: Option<String>,

    #[arg(long = "departpos")]
    pub departpos: Option<String>,

    #[arg(long = "departspeed")]
    pub departspeed: Option<String>,

    #[arg(long = "emissions.volumetric-fuel", default_value_t = String::from("False"))]
    pub emissions_volumetric_fuel: String,

    #[arg(long = "end", short = 'e')]
    pub end: Option<f64>,

    #[arg(long = "error-log")]
    pub error_log: Option<String>,

    #[arg(long = "exit-times", default_value_t = String::from("False"))]
    pub exit_times: String,

    #[arg(long = "gawron.a", default_value = "0.5")]
    pub gawron_a: f64,

    #[arg(long = "gawron.beta", default_value = "0.9")]
    pub gawron_beta: f64,

    #[arg(long = "human-readable-time", short = 'H', default_value_t = String::from("False"))]
    pub human_readable_time: String,

    #[arg(long = "ignore-errors", default_value_t = String::from("False"))]
    pub ignore_errors: String,

    #[arg(long = "intermodal-network-output")]
    pub intermodal_network_output: Option<String>,

    #[arg(long = "intermodal-weight-output")]
    pub intermodal_weight_output: Option<String>,

    #[arg(long = "junction-taz", default_value_t = String::from("False"))]
    pub junction_taz: String,

    #[arg(long = "keep-all-routes", default_value_t = String::from("False"))]
    pub keep_all_routes: String,

    #[arg(long = "keep-route-probability")]
    pub keep_route_probability: Option<f64>,

    #[arg(long = "keep-vtype-distributions", default_value_t = String::from("False"))]
    pub keep_vtype_distributions: String,

    #[arg(long = "lane-weight-files")]
    pub lane_weight_files: Option<String>,

    #[arg(long = "language")]
    pub language: Option<String>,

    #[arg(long = "log", short = 'l')]
    pub log: Option<String>,

    #[arg(long = "log.processid", default_value_t = String::from("False"))]
    pub log_processid: String,

    #[arg(long = "log.timestamps", default_value_t = String::from("False"))]
    pub log_timestamps: String,

    #[arg(long = "logit", default_value_t = String::from("False"))]
    pub logit: String,

    #[arg(long = "logit.beta", default_value = "0.15")]
    pub logit_beta: f64,

    #[arg(long = "logit.gamma", default_value = "1.0")]
    pub logit_gamma: f64,

    #[arg(long = "logit.theta", default_value = "-1.0")]
    pub logit_theta: f64,

    #[arg(long = "mapmatch.distance")]
    pub mapmatch_distance: Option<f64>,

    #[arg(long = "mapmatch.junctions", default_value_t = String::from("False"))]
    pub mapmatch_junctions: String,

    #[arg(long = "mapmatch.taz", default_value_t = String::from("False"))]
    pub mapmatch_taz: String,

    #[arg(long = "max-alternatives", default_value = "5")]
    pub max_alternatives: u32,

    #[arg(long = "message-log")]
    pub message_log: Option<String>,

    #[arg(long = "named-routes", default_value_t = String::from("False"))]
    pub named_routes: String,

    #[arg(long = "net-file", short = 'n')]
    pub net_file: Option<String>,

    #[arg(long = "no-internal-links", default_value_t = String::from("False"))]
    pub no_internal_links: String,

    #[arg(long = "no-step-log", default_value_t = String::from("False"))]
    pub no_step_log: String,

    #[arg(long = "no-warnings", default_value_t = String::from("False"))]
    pub no_warnings: String,

    #[arg(long = "output-file", short = 'o')]
    pub output_file: Option<String>,

    #[arg(long = "output-dir")]
    pub output_dir: Option<String>,

    #[arg(long = "output-prefix")]
    pub output_prefix: Option<String>,

    #[arg(long = "persontrip.ride-public-line", default_value_t = String::from("False"))]
    pub persontrip_ride_public_line: String,

    #[arg(long = "persontrip.taxi.waiting-time")]
    pub persontrip_taxi_waiting_time: Option<f64>,

    #[arg(long = "persontrip.transfer.car-walk")]
    pub persontrip_transfer_car_walk: Option<String>,

    #[arg(long = "persontrip.transfer.taxi-walk")]
    pub persontrip_transfer_taxi_walk: Option<String>,

    #[arg(long = "persontrip.transfer.walk-taxi")]
    pub persontrip_transfer_walk_taxi: Option<String>,

    #[arg(long = "persontrip.walk-opposite-factor")]
    pub persontrip_walk_opposite_factor: Option<f64>,

    #[arg(long = "persontrip.walkfactor")]
    pub persontrip_walkfactor: Option<f64>,

    #[arg(long = "phemlight-path")]
    pub phemlight_path: Option<String>,

    #[arg(long = "phemlight-temperature")]
    pub phemlight_temperature: Option<f64>,

    #[arg(long = "phemlight-year")]
    pub phemlight_year: Option<i32>,

    #[arg(long = "precision")]
    pub precision: Option<i32>,

    #[arg(long = "precision.geo")]
    pub precision_geo: Option<i32>,

    #[arg(long = "print-options", default_value_t = String::from("False"))]
    pub print_options: String,

    #[arg(long = "ptline-routing", default_value_t = String::from("False"))]
    pub ptline_routing: String,

    #[arg(long = "railway.max-train-length")]
    pub railway_max_train_length: Option<f64>,

    #[arg(long = "random", default_value_t = String::from("False"))]
    pub random: String,

    #[arg(long = "randomize-flows", default_value_t = String::from("False"))]
    pub randomize_flows: String,

    #[arg(long = "remove-loops", default_value_t = String::from("False"))]
    pub remove_loops: String,

    #[arg(long = "repair", default_value_t = String::from("False"))]
    pub repair: String,

    #[arg(long = "repair.from", default_value_t = String::from("False"))]
    pub repair_from: String,

    #[arg(long = "repair.to", default_value_t = String::from("False"))]
    pub repair_to: String,

    #[arg(long = "restriction-params")]
    pub restriction_params: Option<String>,

    #[arg(long = "route-choice-method", default_value_t = String::from("gawron"))]
    pub route_choice_method: String,

    #[arg(long = "route-files", short = 't')]
    pub route_files: Option<String>,

    #[arg(long = "route-length", default_value_t = String::from("False"))]
    pub route_length: String,

    #[arg(long = "route-steps", short = 's')]
    pub route_steps: Option<f64>,

    #[arg(long = "routing-algorithm")]
    pub routing_algorithm: Option<String>,

    #[arg(long = "routing-threads")]
    pub routing_threads: Option<i32>,

    #[arg(long = "save-commented", default_value_t = String::from("False"))]
    pub save_commented: String,

    #[arg(long = "save-configuration", short = 'C')]
    pub save_configuration: Option<String>,

    #[arg(long = "save-configuration.relative", default_value_t = String::from("False"))]
    pub save_configuration_relative: String,

    #[arg(long = "save-schema")]
    pub save_schema: Option<String>,

    #[arg(long = "save-template")]
    pub save_template: Option<String>,

    #[arg(long = "scale")]
    pub scale: Option<f64>,

    #[arg(long = "scale-suffix")]
    pub scale_suffix: Option<String>,

    #[arg(long = "seed")]
    pub seed: Option<i32>,

    #[arg(long = "skip-new-routes", default_value_t = String::from("False"))]
    pub skip_new_routes: String,

    #[arg(long = "stats-period")]
    pub stats_period: Option<i32>,

    #[arg(long = "unsorted-input", default_value_t = String::from("False"))]
    pub unsorted_input: String,

    #[arg(long = "verbose", short = 'v', default_value_t = String::from("False"))]
    pub verbose: String,

    #[arg(long = "vtype-output")]
    pub vtype_output: Option<String>,

    #[arg(long = "weight-attribute", short = 'x')]
    pub weight_attribute: Option<String>,

    #[arg(long = "weight-files", short = 'w')]
    pub weight_files: Option<String>,

    #[arg(long = "weight-period")]
    pub weight_period: Option<f64>,

    #[arg(long = "weights.expand", default_value_t = String::from("False"))]
    pub weights_expand: String,

    #[arg(long = "weights.interpolate", default_value_t = String::from("False"))]
    pub weights_interpolate: String,

    #[arg(long = "weights.minor-penalty")]
    pub weights_minor_penalty: Option<f64>,

    #[arg(long = "weights.priority-factor")]
    pub weights_priority_factor: Option<f64>,

    #[arg(long = "weights.random-factor")]
    pub weights_random_factor: Option<f64>,

    #[arg(long = "weights.tls-penalty")]
    pub weights_tls_penalty: Option<f64>,

    #[arg(long = "weights.turnaround-penalty")]
    pub weights_turnaround_penalty: Option<f64>,

    #[arg(long = "with-taz", default_value_t = String::from("False"))]
    pub with_taz: String,

    #[arg(long = "write-costs", default_value_t = String::from("False"))]
    pub write_costs: String,

    #[arg(long = "write-license", default_value_t = String::from("False"))]
    pub write_license: String,

    #[arg(long = "write-trips", default_value_t = String::from("False"))]
    pub write_trips: String,

    #[arg(long = "write-trips.geo", default_value_t = String::from("False"))]
    pub write_trips_geo: String,

    #[arg(long = "write-trips.junctions", default_value_t = String::from("False"))]
    pub write_trips_junctions: String,

    #[arg(long = "xml-validation", short = 'X')]
    pub xml_validation: Option<String>,

    #[arg(long = "xml-validation.net")]
    pub xml_validation_net: Option<String>,

    #[arg(long = "xml-validation.routes")]
    pub xml_validation_routes: Option<String>,

    /// the directory containing the files where the cch folder and the queries will be read from
    /// default is the current working directory
    #[arg(long = "input-dir", default_value_t = String::from(env::current_dir().unwrap().to_str().unwrap()))]
    pub input_dir: String,

    /// the files `<input-prefix>_{iteration:3>0}.rou.xml`, `<input-prefix>_{iteration:3>0}.rou.alt.xml` will be read as input
    #[arg(long = "input-prefix", default_value = "")]
    pub input_prefix: String,

    /// the iteration number to use for the input files
    #[arg(long = "iteration")]
    pub iteration: u32,

    /// if set skips writing the alternative routes file in the sumo format
    #[arg(long = "no-write-sumo-alternatives", default_value_t = String::from("False"))]
    pub no_write_sumo_alternatives: String,
}

impl RouterArgs {
    pub fn get_choice_algorithm(&self) -> choice::ChoiceAlgorithm {
        match self.route_choice_method.as_str() {
            choice::LOGIT => {
                println!(
                    "Using logit with beta={}, gamma={}, theta={}",
                    self.logit_beta, self.logit_gamma, self.logit_theta
                );
                choice::ChoiceAlgorithm::create_logit(self.logit_beta, self.logit_gamma, self.logit_theta)
            }
            choice::GAWRON => {
                println!("Using gawron with a={}, beta={}", self.gawron_a, self.gawron_beta);
                choice::ChoiceAlgorithm::create_gawron(self.gawron_a, self.gawron_beta)
            }
            _ => panic!("Unknown choice algorithm: {}", self.route_choice_method),
        }
    }

    pub fn get_write_sumo_alternatives(&self) -> bool {
        self.no_write_sumo_alternatives == "false" || self.no_write_sumo_alternatives == "0" || self.no_write_sumo_alternatives == "False"
    }
}

/// For FastDta add additional arguments:

#[derive(Parser, Debug)]
#[command(version, about = "fastdta routing CLI options", long_about = None)]
pub struct FastDtaArgs {
    #[command(flatten)]
    pub router_args: RouterArgs,

    ///sample sizes and number of sample per iteration
    /// e.g. --samples "0.1 0.2 0.3 0.4" will sample 10% in the first iteration, 20% in the second, 30% in the third, and 40% in the fourth,
    /// such that each sample does not intersect with previous samples
    /// Samples are uniformly distributed over all trips
    /// Format: "<f64>[ <f64>]*" - space-separated floating point values
    #[arg(long = "samples")]
    pub samples: Option<String>,

    /// sets the VDF
    #[arg(long = "traffic-model", default_value = "modified-lee")]
    pub traffic_model: String,

    /// threshold for the number of data points (lane density and speed observations) per edge
    /// to calibrate the traffic model parameters
    #[arg(long = "calibration-data-points-threshold", default_value = "1000")]
    pub calibration_data_points_threshold: usize,
}

impl FastDtaArgs {
    pub fn get_traffic_model(&self) -> TrafficModelType {
        match self.traffic_model.as_str() {
            "modified-lee" => {
                println!("Using modified-lee as traffic model");
                TrafficModelType::ModifiedLee
            }
            _ => panic!("Unknown VDF type: {}", self.traffic_model),
        }
    }

    pub fn get_samples(&self) -> Vec<f64> {
        let smpls = match &self.samples {
            Some(s) => {
                // Parse space-separated f64 values from string
                s.split_whitespace()
                    .map(|val| val.parse::<f64>().expect(&format!("Invalid sample value: {}", val)))
                    .collect()
            }
            None => vec![0.9, 0.1],
        };

        println!("Using samples: {:?}", smpls);
        smpls
    }
}
