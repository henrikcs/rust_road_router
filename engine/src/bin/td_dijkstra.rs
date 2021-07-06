// TD Dijkstra baseline.

use std::{env, error::Error, path::Path};
#[macro_use]
extern crate rust_road_router;
use rust_road_router::{
    algo::{
        dijkstra::query::{dijkstra::Server, td_dijkstra::TDDijkstraOps},
        *,
    },
    cli::CliErr,
    datastr::graph::time_dependent::*,
    datastr::graph::*,
    io::*,
    report::*,
};

use time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    let _reporter = enable_reporting("td_dijkstra");

    let arg = &env::args().skip(1).next().ok_or(CliErr("No directory arg given"))?;
    let path = Path::new(arg);

    let first_out = Vec::load_from(path.join("first_out"))?;
    let head = Vec::load_from(path.join("head"))?;
    let first_ipp_of_arc = Vec::load_from(path.join("first_ipp_of_arc"))?;
    let ipp_departure_time = Vec::<u32>::load_from(path.join("ipp_departure_time"))?;
    let ipp_travel_time = Vec::load_from(path.join("ipp_travel_time"))?;

    report!("unprocessed_graph", { "num_nodes": first_out.len() - 1, "num_arcs": head.len(), "num_ipps": ipp_departure_time.len() });

    let graph = TDGraph::new(first_out, head, first_ipp_of_arc, ipp_departure_time, ipp_travel_time);

    report!("graph", { "num_nodes": graph.num_nodes(), "num_arcs": graph.num_arcs(), "num_ipps": graph.num_ipps(), "num_constant_ttfs": graph.num_constant() });

    let mut algo_runs_ctxt = push_collection_context("algo_runs".to_string());

    let mut server = Server::<_, TDDijkstraOps>::new(graph);

    let mut query_dir = None;
    let mut base_dir = Some(path);

    while let Some(base) = base_dir {
        if base.join("uniform_queries").exists() {
            query_dir = Some(base.join("uniform_queries"));
            break;
        } else {
            base_dir = base.parent();
        }
    }

    let core_ids = core_affinity::get_core_ids().unwrap();
    core_affinity::set_for_current(core_ids[0]);

    if let Some(path) = query_dir {
        let from = Vec::load_from(path.join("source_node"))?;
        let at = Vec::<u32>::load_from(path.join("source_time"))?;
        let to = Vec::load_from(path.join("target_node"))?;

        let num_queries = 1000;

        let mut total_time = Duration::zero();

        for ((from, to), at) in from.into_iter().zip(to.into_iter()).zip(at.into_iter()).take(num_queries) {
            let _query_ctxt = algo_runs_ctxt.push_collection_item();
            report!("from", from);
            report!("to", to);
            let (ea, time) = measure(|| server.td_query(TDQuery { from, to, departure: at }).map(|res| res.distance() + at));
            report!("running_time_ms", time.to_std().unwrap().as_nanos() as f64 / 1_000_000.0);
            report!("result", ea.unwrap_or(INFINITY));
            total_time = total_time + time;
        }
        eprintln!("TD Dijkstra {}", total_time / (num_queries as i32));
    }

    Ok(())
}
