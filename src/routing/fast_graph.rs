use serde_derive::{Deserialize, Serialize};

use super::{
    edge::{DirectedHeadlessWeightedEdge, DirectedTaillessWeightedEdge},
    fast_edge_access::{FastInEdgeAccess, FastOutEdgeAccess},
    graph::Graph,
    naive_graph::NaiveGraph,
    route::{Route, RouteRequest},
    types::VertexId,
};

#[derive(Clone)]
/// Gives fast access to predecessor and successor in a graph.
/// As FastGraph uses FastEdgeAccess, an out_edges head is acutally its tail.
pub struct FastGraph {
    pub num_nodes: u32,
    pub out_edges: FastOutEdgeAccess,
    pub in_edges: FastInEdgeAccess,
}

impl FastGraph {
    pub fn from_graph(graph: &Graph) -> FastGraph {
        let num_nodes = graph.in_edges.len() as u32;

        let out_edges: Vec<_> = graph.out_edges.iter().flatten().cloned().collect();
        let out_edges = FastOutEdgeAccess::new(&out_edges);

        let in_edges: Vec<_> = graph.in_edges.iter().flatten().cloned().collect();
        let in_edges = FastInEdgeAccess::new(&in_edges);

        FastGraph {
            num_nodes,
            out_edges,
            in_edges,
        }
    }
    pub fn out_edges(&self, source: VertexId) -> &[DirectedTaillessWeightedEdge] {
        self.out_edges.edges(source)
    }

    pub fn in_edges(&self, target: VertexId) -> &[DirectedHeadlessWeightedEdge] {
        self.in_edges.edges(target)
    }

    pub fn from_naive_graph(graph: &NaiveGraph) -> FastGraph {
        let out_edges = FastOutEdgeAccess::new(&graph.edges);
        let in_edges = FastInEdgeAccess::new(&graph.edges);

        FastGraph {
            num_nodes: graph.nodes.len() as u32,
            out_edges,
            in_edges,
        }
    }
}
