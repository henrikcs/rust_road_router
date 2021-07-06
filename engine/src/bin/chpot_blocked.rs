#[macro_use]
extern crate rust_road_router;
use rust_road_router::{cli::CliErr, datastr::graph::*, io::*, report::*};
use std::{env, error::Error, path::Path};

const TUNNEL_BIT: u8 = 1;
const FREEWAY_BIT: u8 = 2;

fn main() -> Result<(), Box<dyn Error>> {
    let _reporter = enable_reporting("chpot_blocked");
    let arg = &env::args().skip(1).next().ok_or(CliErr("No graph directory arg given"))?;
    let path = Path::new(arg);

    let arc_category = Vec::<u8>::load_from(path.join("arc_category"))?;

    let mut exps_ctxt = push_collection_context("experiments".to_string());

    {
        let _exp_ctx = exps_ctxt.push_collection_item();
        report!("experiment", "no_tunnels");

        rust_road_router::experiments::chpot::run(path, |_graph, _rng, travel_time| {
            for (weight, &category) in travel_time.iter_mut().zip(arc_category.iter()) {
                if (category & TUNNEL_BIT) != 0 {
                    *weight = INFINITY;
                }
            }

            Ok(())
        })?;
    }

    {
        let _exp_ctx = exps_ctxt.push_collection_item();
        report!("experiment", "no_highways");

        rust_road_router::experiments::chpot::run(path, |_graph, _rng, travel_time| {
            for (weight, &category) in travel_time.iter_mut().zip(arc_category.iter()) {
                if (category & FREEWAY_BIT) != 0 {
                    *weight = INFINITY;
                }
            }

            Ok(())
        })?;
    }

    Ok(())
}
