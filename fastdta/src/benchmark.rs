use std::{fs, io::Write};

pub fn write_result(benchmark_out_dir: &String, result: &String) {
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

    writeln!(file, "{}", result).unwrap();
    file.flush().unwrap();
}
