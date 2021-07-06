// CATCHUp utilitie to write RoutingKit querties to KaTCH demands

use std::{env, error::Error, fs::File, io::prelude::*, path::Path};

use rust_road_router::{cli::CliErr, datastr::graph::INFINITY, io::*};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let arg = &args.next().ok_or(CliErr("No directory arg given"))?;
    let path = Path::new(arg);

    let ground_truth = Vec::<u32>::load_from(path.join("optimal_target_time"))?;

    let mut query_dir = None;
    let mut base_dir = Some(path);

    while let Some(base) = base_dir {
        if base.join("source_node").exists() {
            query_dir = Some(base);
            break;
        } else {
            base_dir = base.parent();
        }
    }

    let query_dir = query_dir.ok_or(CliErr("No queries found"))?;

    let from = Vec::<u32>::load_from(query_dir.join("source_node"))?;
    let at = Vec::<u32>::load_from(query_dir.join("source_time"))?;
    let to = Vec::<u32>::load_from(query_dir.join("target_node"))?;
    let rank = Vec::<u32>::load_from(query_dir.join("dij_rank"))?;

    let output = &args.next().ok_or(CliErr("No output file given"))?;
    let mut output = File::create(output)?;

    output.write_all(b"demands\r\n")?;
    let num_queries = ground_truth.iter().filter(|&&t| t != INFINITY).count();
    output.write_all(&(num_queries as u32).to_ne_bytes())?;

    for i in 0..ground_truth.len() {
        if ground_truth[i] != INFINITY {
            output.write_all(&from[i].to_ne_bytes())?;
            output.write_all(&to[i].to_ne_bytes())?;
            let t = f64::from(at[i]) / 100.0;
            output.write_all(&t.to_bits().to_ne_bytes())?;
            let t = f64::from(ground_truth[i]) / 100.0;
            output.write_all(&t.to_bits().to_ne_bytes())?;
            output.write_all(&rank[i].to_ne_bytes())?;
        }
    }

    output.write_all(&0x0716_2534u32.to_ne_bytes())?;

    Ok(())
}
