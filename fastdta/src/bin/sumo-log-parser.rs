use anyhow::{Context, Result, anyhow};
use regex::Regex;
use serde::Serialize;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Default, Clone)]
struct PhaseTiming {
    begin_time: Option<String>,
    end_time: Option<String>,
    duration: Option<String>,
}

#[derive(Debug, Serialize, Default, Clone)]
struct Step {
    iteration: u64,
    router: Option<PhaseTiming>,
    simulation: Option<PhaseTiming>,
    duration: Option<String>, // iteration duration
    relative_travel_time_deviation: Option<String>,
}

#[derive(Debug, Serialize, Default, Clone)]
struct OutputDirMeta {
    full: String,
    path_root: Option<String>,
    timestamp: Option<String>,             // YYYY-MM-DD-HH-MM
    experiments_file_line: Option<String>, // experiments-file-line
    algorithm: Option<String>,             // last path segment
}

#[derive(Debug, Serialize, Default, Clone)]
struct PhaseLine {
    executable: String,
    output_dir: String, // keep raw path to keep JSON lean
    iteration: i64,     // can be -1 for preprocessing
    algorithm_phase: String,
    phase_duration: String,
}

#[derive(Debug, Serialize, Default, Clone)]
struct Experiment {
    algorithm: Option<String>,
    aggregation: Option<String>,
    output_dir: Option<OutputDirMeta>,
    steps: Vec<Step>,
    phases: Vec<PhaseLine>,
    total_duration: Option<String>, // from "dua-iterate ended (duration: ...)"
}

#[derive(Debug, Serialize, Default)]
struct LogSummary {
    experiments: Vec<Experiment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SubPhase {
    Router,
    Simulation,
}

fn clean_brackets(s: &str) -> String {
    s.trim().trim_matches('<').trim_matches('>').to_string()
}

fn parse_output_dir_meta(path: &str) -> OutputDirMeta {
    let cleaned = clean_brackets(path);
    let parts: Vec<&str> = cleaned.split('/').filter(|p| !p.is_empty()).collect();

    let mut meta = OutputDirMeta {
        full: cleaned.clone(),
        path_root: None,
        timestamp: None,
        experiments_file_line: None,
        algorithm: None,
    };

    if parts.len() >= 3 {
        let alg = parts[parts.len() - 1].to_string();
        let exp_line = parts[parts.len() - 2].to_string();
        let ts = parts[parts.len() - 3].to_string();
        let ts_re = Regex::new(r"^\d{4}-\d{2}-\d{2}-\d{2}-\d{2}$").unwrap();
        if ts_re.is_match(&ts) {
            meta.algorithm = Some(alg);
            meta.experiments_file_line = Some(exp_line);
            meta.timestamp = Some(ts.clone());
            let root = parts[..parts.len() - 3].join("/");
            meta.path_root = Some(if cleaned.starts_with('/') { format!("/{}", root) } else { root });
        }
    }
    meta
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input_log_file> <output_json_file>", args[0]);
        std::process::exit(2);
    }
    let input = &args[1];
    let output = &args[2];

    let file = File::open(Path::new(input)).with_context(|| format!("Failed to open input file: {}", input))?;
    let reader = BufReader::new(file);

    // Experiment header (now only 2 lines)
    let re_experiment_start = Regex::new(r#"^\s*Processing with algorithm:\s*(?P<alg>[^,]+),\s*aggregation:\s*(?P<agg>.+?)\s*$"#)?;
    let re_output_dir = Regex::new(r#"^\s*Output directory:\s*(?P<dir>.+?)\s*$"#)?;

    // Experiment end
    let re_experiment_end = Regex::new(r#"^\s*dua-iterate ended\s*\(duration:\s*(?P<dur>.+?)\)\s*$"#)?;

    // Semicolon “phase” lines:
    // <executable>; <path>; <iteration>; <algorithm-phase>; <phase-duration>
    // (allow negative iteration like -1)
    let re_exec_semicolon = Regex::new(r#"^\s*(?P<exe>[^;]+);\s*(?P<odir>[^;]+);\s*(?P<iter>-?\d+);\s*(?P<phase>[^;]+);\s*(?P<dur>[^;]+)\s*$"#)?;

    // Step scaffolding
    let re_step_start = Regex::new(r#"^\s*>\s*Executing step\s+(?P<iter>\d+)\s*$"#)?;
    let re_running_router = Regex::new(r#"^\s*>>\s*Running router\b"#)?;
    let re_running_sim = Regex::new(r#"^\s*>>\s*Running simulation\b"#)?;
    let re_begin = Regex::new(r#"^\s*>>>\s*Begin time:\s*(?P<t>.+?)\s*$"#)?;
    let re_end = Regex::new(r#"^\s*>>>\s*End time:\s*(?P<t>.+?)\s*$"#)?;
    let re_duration = Regex::new(r#"^\s*>>>\s*Duration:\s*(?P<d>.+?)\s*$"#)?;
    let re_block_end = Regex::new(r#"^\s*<<\s*$"#)?;

    // relative travel time deviation line (appears after simulation block)
    // Example: "< relative travel time deviation in the last ... steps: <deviation>"
    let re_rel_dev = Regex::new(r#"^\s*<\s*relative travel time deviation in the last .*? steps:\s*(?P<dev>.+?)\s*$"#)?;

    // Step end
    let re_step_end = Regex::new(r#"^\s*<\s*Step\s+(?P<iter>\d+)\s*ended\s*\(duration:\s*(?P<dur>.+?)\)\s*$"#)?;

    let mut summary = LogSummary::default();
    let mut current_exp: Option<Experiment> = None;
    let mut current_step: Option<Step> = None;
    let mut current_subphase: Option<SubPhase> = None;

    let mut router_tmp = PhaseTiming::default();
    let mut sim_tmp = PhaseTiming::default();

    for line in reader.lines() {
        let line = line?;

        // Experiment start
        if let Some(caps) = re_experiment_start.captures(&line) {
            // flush any open experiment
            if let Some(exp) = current_exp.take() {
                summary.experiments.push(exp);
            }
            let mut exp = Experiment::default();
            exp.algorithm = Some(caps["alg"].trim().to_string());
            exp.aggregation = Some(caps["agg"].trim().to_string());
            current_exp = Some(exp);
            continue;
        }

        // Output directory for the current experiment
        if let Some(caps) = re_output_dir.captures(&line) {
            if let Some(ref mut exp) = current_exp {
                let dir = caps["dir"].trim().to_string();
                exp.output_dir = Some(parse_output_dir_meta(&dir));
            }
            continue;
        }

        // Semicolon phase line (flat)
        if let Some(caps) = re_exec_semicolon.captures(&line) {
            if let Some(ref mut exp) = current_exp {
                let iteration: i64 = caps["iter"].trim().parse().unwrap_or(0);
                exp.phases.push(PhaseLine {
                    executable: caps["exe"].trim().to_string(),
                    output_dir: caps["odir"].trim().to_string(),
                    iteration,
                    algorithm_phase: caps["phase"].trim().to_string(),
                    phase_duration: caps["dur"].trim().to_string(),
                });
            }
            continue;
        }

        // Step start
        if let Some(caps) = re_step_start.captures(&line) {
            if let Some(ref mut exp) = current_exp {
                if let Some(st) = current_step.take() {
                    exp.steps.push(st);
                }
            }
            current_step = Some(Step {
                iteration: caps["iter"].trim().parse().unwrap_or(0),
                ..Default::default()
            });
            current_subphase = None;
            router_tmp = PhaseTiming::default();
            sim_tmp = PhaseTiming::default();
            continue;
        }

        // Enter subphases
        if re_running_router.is_match(&line) {
            current_subphase = Some(SubPhase::Router);
            continue;
        }
        if re_running_sim.is_match(&line) {
            current_subphase = Some(SubPhase::Simulation);
            continue;
        }

        // Timings inside a subphase
        if let Some(caps) = re_begin.captures(&line) {
            match current_subphase {
                Some(SubPhase::Router) => router_tmp.begin_time = Some(caps["t"].trim().to_string()),
                Some(SubPhase::Simulation) => sim_tmp.begin_time = Some(caps["t"].trim().to_string()),
                None => {}
            }
            continue;
        }
        if let Some(caps) = re_end.captures(&line) {
            match current_subphase {
                Some(SubPhase::Router) => router_tmp.end_time = Some(caps["t"].trim().to_string()),
                Some(SubPhase::Simulation) => sim_tmp.end_time = Some(caps["t"].trim().to_string()),
                None => {}
            }
            continue;
        }
        if let Some(caps) = re_duration.captures(&line) {
            match current_subphase {
                Some(SubPhase::Router) => router_tmp.duration = Some(caps["d"].trim().to_string()),
                Some(SubPhase::Simulation) => sim_tmp.duration = Some(caps["d"].trim().to_string()),
                None => {}
            }
            continue;
        }

        // End of router/simulation block
        if re_block_end.is_match(&line) {
            if let Some(phase) = current_subphase {
                if let Some(ref mut st) = current_step {
                    match phase {
                        SubPhase::Router => {
                            if st.router.is_none() {
                                st.router = Some(router_tmp.clone());
                            }
                        }
                        SubPhase::Simulation => {
                            if st.simulation.is_none() {
                                st.simulation = Some(sim_tmp.clone());
                            }
                        }
                    }
                }
            }
            current_subphase = None;
            continue;
        }

        // New: relative travel time deviation line
        if let Some(caps) = re_rel_dev.captures(&line) {
            if let Some(ref mut st) = current_step {
                st.relative_travel_time_deviation = Some(caps["dev"].trim().to_string());
            }
            continue;
        }

        // Step end
        if let Some(caps) = re_step_end.captures(&line) {
            if let Some(ref mut st) = current_step {
                let _ended_iter: u64 = caps["iter"].trim().parse().unwrap_or(0);
                st.duration = Some(caps["dur"].trim().to_string());
            }
            if let Some(ref mut exp) = current_exp {
                if let Some(st) = current_step.take() {
                    exp.steps.push(st);
                }
            }
            // reset temps
            router_tmp = PhaseTiming::default();
            sim_tmp = PhaseTiming::default();
            current_subphase = None;
            continue;
        }

        // Experiment end
        if let Some(caps) = re_experiment_end.captures(&line) {
            if let Some(mut exp) = current_exp.take() {
                exp.total_duration = Some(caps["dur"].trim().to_string());
                if let Some(st) = current_step.take() {
                    exp.steps.push(st);
                }
                summary.experiments.push(exp);
            }
            // reset state
            current_step = None;
            current_subphase = None;
            router_tmp = PhaseTiming::default();
            sim_tmp = PhaseTiming::default();
            continue;
        }

        // Non-matching: ignore
    }

    // Flush if file ended mid-experiment
    if let Some(mut exp) = current_exp.take() {
        if let Some(st) = current_step.take() {
            exp.steps.push(st);
        }
        summary.experiments.push(exp);
    }

    // Write JSON
    let out_file = File::create(Path::new(output)).with_context(|| format!("Failed to create output file: {}", output))?;
    serde_json::to_writer_pretty(out_file, &summary).map_err(|e| anyhow!("Failed to write JSON: {}", e))?;

    Ok(())
}
