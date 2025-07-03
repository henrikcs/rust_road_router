use conversion::sumo::XmlWriter;
use conversion::sumo::routes::RoutesDocumentRoot;
use conversion::sumo::routes_writer::{self, SumoRoutesWriter};
use fastdta::benchmark::write_result;
use fastdta::cli;
use fastdta::cli::Parser;
use std::thread;
use std::time::Instant;

fn main() {
    let args = cli::Args::parse();

    let Some(input_prefix) = args.input_prefix else {
        panic!("No input prefix provided to read files from. Use --input-prefix <prefix> (or -i <prefix) to specify a inputs file.");
    };

    let Some(_demand_file) = args.route_files else {
        panic!("No route file(s) provided. Use --route-files <path> to specify route file(s)");
    };

    let Some(output_file) = args.output_file else {
        panic!("No output file provided. Use --output-file <path> (or -o <path>) to specify the output file.");
    };

    println!("Starting timer");
    let timer = Instant::now();

    thread::sleep(std::time::Duration::from_secs(1));

    // SumoRoutesWriter::write(output_file.as_str(), RoutesDocumentRoot {Default::default()}).unwrap();

    let nanos = timer.elapsed().as_nanos();

    println!("Calculation took {} nanoseconds", nanos);

    write_result(&String::from("../../../out"), &nanos.to_string());

    // let graph = SumoGraphReader::read(&network_file).unwrap();

    // let astar_algo = Astar {
    //     start: String::from("1000054341"),
    //     end: String::from("1000085895"),
    // };

    // let path = astar_algo.run(&graph);

    // println!("{path:?}");
}
