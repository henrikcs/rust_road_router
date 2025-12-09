use super::*;
use crate::algo::dijkstra::generic_dijkstra::*;
use crate::datastr::graph::floating_time_dependent::*;

pub struct Server<'a> {
    graph: &'a TDGraph,
    data: DijkstraData<Timestamp, ()>,
}

impl<'a> Server<'a> {
    pub fn new(graph: &TDGraph) -> Server<'_> {
        Server {
            data: DijkstraData::new(graph.num_nodes()),
            graph,
        }
    }

    pub fn ranks<F>(&mut self, from: NodeId, departure_time: Timestamp, mut callback: F)
    where
        F: (FnMut(NodeId, Timestamp, usize)),
    {
        let mut ops = FlTDDijkstraOps();
        let mut dijkstra = DijkstraRun::query(
            self.graph,
            &mut self.data,
            &mut ops,
            DijkstraInit {
                source: NodeIdT(from),
                initial_state: departure_time,
            },
        );

        let mut i: usize = 0;
        while let Some(node) = dijkstra.next() {
            i += 1;
            if (i & (i - 1)) == 0 {
                // power of two
                callback(node, *dijkstra.tentative_distance(node), i.trailing_zeros() as usize);
            }
        }
    }

    fn distance(&mut self, query: TDQuery<Timestamp>) -> Option<FlWeight> {
        report!("algo", "Floating TD-Dijkstra");
        let mut ops = FlTDDijkstraOps();
        let mut dijkstra = DijkstraRun::query(self.graph, &mut self.data, &mut ops, DijkstraInit::from_query(&query));

        while let Some(node) = dijkstra.next() {
            if node == query.to {
                return Some(*dijkstra.tentative_distance(node) - query.departure);
            }
        }

        None
    }

    fn path(&self, query: TDQuery<Timestamp>) -> Vec<(NodeId, Timestamp)> {
        let mut path = vec![(query.to, self.data.distances[query.to as usize])];

        while path.last().unwrap().0 != query.from {
            let next = self.data.predecessors[path.last().unwrap().0 as usize].0;
            let t = self.data.distances[next as usize];
            path.push((next, t));
        }

        path.reverse();
        path
    }

    fn edge_path(&self, query: TDQuery<Timestamp>) -> Vec<EdgeIdT> {
        let node_path = self.path(query).iter().map(|(node, _)| *node).collect::<Vec<_>>();
        let mut edge_path = Vec::with_capacity(node_path.len() - 1);
        for i in 0..node_path.len() - 1 {
            let from = node_path[i];
            let to = node_path[i + 1];
            let edge = self
                .graph
                .edge_indices(from, to)
                .next()
                .unwrap_or_else(|| panic!("No edge from {} to {} in original graph, path: {:?}", from, to, node_path));
            edge_path.push(edge);
        }

        edge_path
    }
}

pub struct PathServerWrapper<'s>(&'s Server<'s>, TDQuery<Timestamp>);

impl<'s> PathServer for PathServerWrapper<'s> {
    type NodeInfo = (NodeId, Timestamp);
    type EdgeInfo = EdgeIdT;

    fn reconstruct_node_path(&mut self) -> Vec<Self::NodeInfo> {
        Server::path(self.0, self.1)
    }
    fn reconstruct_edge_path(&mut self) -> Vec<Self::EdgeInfo> {
        Server::edge_path(self.0, self.1)
    }
}

impl TDQueryServer<Timestamp, FlWeight> for Server<'_> {
    type P<'s>
        = PathServerWrapper<'s>
    where
        Self: 's;

    fn td_query(&mut self, query: TDQuery<Timestamp>) -> QueryResult<Self::P<'_>, FlWeight> {
        QueryResult::new(self.distance(query), PathServerWrapper(self, query))
    }
}

#[derive(Default)]
struct FlTDDijkstraOps();

impl DijkstraOps<TDGraph> for FlTDDijkstraOps {
    type Label = Timestamp;
    type LinkResult = Timestamp;
    type Arc = (NodeIdT, EdgeIdT);
    type PredecessorLink = ();

    #[inline(always)]
    fn link(&mut self, graph: &TDGraph, _parents: &[(NodeId, Self::PredecessorLink)], _tail: NodeIdT, label: &Timestamp, link: &Self::Arc) -> Self::LinkResult {
        *label + graph.travel_time_function(link.1 .0).evaluate(*label)
    }

    #[inline(always)]
    fn merge(&mut self, label: &mut Timestamp, linked: Self::LinkResult) -> bool {
        if linked < *label {
            *label = linked;
            return true;
        }
        false
    }

    #[inline(always)]
    fn predecessor_link(&self, _link: &Self::Arc) -> Self::PredecessorLink {}
}

impl Reset for Timestamp {
    const DEFAULT: Self = Self::NEVER;
}

impl Label for Timestamp {
    type Key = Self;

    fn neutral() -> Self {
        Timestamp::NEVER
    }

    #[inline(always)]
    fn key(&self) -> Self::Key {
        *self
    }
}
