use rust_road_router::{
    algo::customizable_contraction_hierarchy::{CCH, ftd_cch},
    datastr::graph::floating_time_dependent::{CustomizedGraph, TDGraph},
};
pub fn customize<'a>(cch: &'a CCH, graph: &'a TDGraph) -> CustomizedGraph<'a> {
    // let _cch_customization_ctxt = algo_runs_ctxt.push_collection_item();
    // customize the cch with the given graph having new travel time functions for each edge
    ftd_cch::customize(&cch, &graph)
}
