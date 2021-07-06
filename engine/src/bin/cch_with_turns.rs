// Example of complete CCH toolchain with turn costs.
// Takes a directory as argument, which has to contain the graph (in RoutingKit format),
// a nested disection order for the turn expanded graph and queries.

use std::{env, error::Error, path::Path};

use rust_road_router::{
    algo::{
        customizable_contraction_hierarchy::{query::Server, *},
        *,
    },
    cli::CliErr,
    datastr::{graph::*, node_order::NodeOrder},
    io::Load,
    report::benchmark::report_time,
};

fn main() -> Result<(), Box<dyn Error>> {
    let arg = &env::args().skip(1).next().ok_or(CliErr("No directory arg given"))?;
    let path = Path::new(arg);

    let first_out = Vec::load_from(path.join("first_out"))?;
    let head = Vec::load_from(path.join("head"))?;
    let travel_time = Vec::load_from(path.join("travel_time"))?;

    let graph = FirstOutGraph::new(&first_out[..], &head[..], &travel_time[..]);
    let graph = graph.line_graph(|_edge1_idx, _edge2_idx| Some(0));

    // use InertialFlowCutter with edge order (cut based) and separator reordering to obtain
    let cch_order = Vec::load_from(path.join("cch_exp_perm"))?;
    let cch_order = NodeOrder::from_node_order(cch_order);

    let cch = contract(&graph, cch_order);
    let cch_order = CCHReordering {
        cch: &cch,
        latitude: &[],
        longitude: &[],
    }
    .reorder_for_seperator_based_customization();
    let cch = contract(&graph, cch_order);
    let cch = cch.into_directed_cch();

    let mut server = Server::new(customize_directed(&cch, &graph));

    let from = Vec::load_from(path.join("test/exp_source"))?;
    let to = Vec::load_from(path.join("test/exp_target"))?;
    let ground_truth = Vec::load_from(path.join("test/exp_travel_time_length"))?;

    report_time("10000 CCH queries", || {
        for ((&from, &to), &ground_truth) in from.iter().zip(to.iter()).zip(ground_truth.iter()).take(10000) {
            let ground_truth = match ground_truth {
                INFINITY => None,
                val => Some(val),
            };

            let mut result = server.query(Query { from, to });
            assert_eq!(result.as_ref().map(|res| res.distance()), ground_truth);
            result.as_mut().map(|res| res.path());
        }
    });

    Ok(())
}
