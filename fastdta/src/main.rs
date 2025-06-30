use conversion::sumo::routes_writer::{RoutesWriter, SumoRoutesWriter};
use fastdta::cli;
use fastdta::cli::Parser;
use std::fs;
use std::io::Write;
use std::thread;
use std::time::Instant;

fn main() {
    let args = cli::Args::parse();

    let Some(_network_file) = args.net_file else {
        panic!("No network file provided. Use --net-file <path> to specify a network file.");
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

    SumoRoutesWriter::write(output_file.as_str(), vec![]).unwrap();

    let nanos = timer.elapsed().as_nanos();

    println!("Calculation took {} nanoseconds", nanos);

    let benchmark_out_dir = "../../../out";
    let mut bechmark_out_file = String::from(benchmark_out_dir);
    bechmark_out_file.push_str("/benchmark.txt");

    // save the current millis as a new line in the output file ./out/benchmark.txt
    // if the file does not exist, create it
    fs::create_dir_all(benchmark_out_dir).unwrap();

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&bechmark_out_file)
        .unwrap_or_else(|e| {
            // If the file cannot be opened, create a new one
            eprintln!("Error opening file: {e}");
            fs::File::create(&bechmark_out_file).unwrap_or_else(|e| {
                eprintln!("Error creating file: {e}");
                panic!("Could not create benchmark file");
            })
        });

    writeln!(file, "{}", nanos).unwrap();
    file.flush().unwrap();

    // let graph = SumoGraphReader::read(&network_file).unwrap();

    // let astar_algo = Astar {
    //     start: String::from("1000054341"),
    //     end: String::from("1000085895"),
    // };

    // let path = astar_algo.run(&graph);

    // println!("{path:?}");
}
