// CATCHUp graph conversion utility from RoutingKit format to KaTCH tpgr.

use std::{env, error::Error, fs::File, io::prelude::*, path::Path};

use rust_road_router::{cli::CliErr, io::*};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let arg = &args.next().ok_or(CliErr("No directory arg given"))?;
    let path = Path::new(arg);

    let period = f64::from(Vec::<u32>::load_from(path.join("period"))?[0]);
    let first_out = Vec::<u32>::load_from(path.join("first_out"))?;
    let head = Vec::<u32>::load_from(path.join("head"))?;
    let first_ipp_of_arc = Vec::<u32>::load_from(path.join("first_ipp_of_arc"))?;
    let ipp_departure_time = Vec::<u32>::load_from(path.join("ipp_departure_time"))?;
    let ipp_travel_time = Vec::<u32>::load_from(path.join("ipp_travel_time"))?;

    let node_count = first_out.len() - 1;
    let arc_count = head.len();
    let ipp_count = ipp_travel_time.len();

    let output = &args.next().ok_or(CliErr("No output file given"))?;
    let mut output = File::create(output)?;
    writeln!(output, "{} {} {} 864000", node_count, arc_count, ipp_count)?;

    for (node, edge_ids) in first_out.windows(2).enumerate() {
        for edge_id in edge_ids[0]..edge_ids[1] {
            let edge_id = edge_id as usize;
            write!(
                output,
                "{} {} {}",
                node,
                head[edge_id],
                first_ipp_of_arc[edge_id + 1] - first_ipp_of_arc[edge_id]
            )?;
            for ipp_idx in first_ipp_of_arc[edge_id]..first_ipp_of_arc[edge_id + 1] {
                let ipp_idx = ipp_idx as usize;
                write!(
                    output,
                    " {} {}",
                    (f64::from(ipp_departure_time[ipp_idx]) * 864_000.0) / period,
                    (f64::from(ipp_travel_time[ipp_idx]) * 864_000.0) / period
                )?;
            }
            writeln!(output)?;
        }
    }

    Ok(())
}
