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
    pub fn out_edges(&self, source: u32) -> &[DirectedTaillessWeightedEdge] {
        self.out_edges.edges(source)
    }

    pub fn in_edges(&self, target: u32) -> &[DirectedHeadlessWeightedEdge] {
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

    /// Check if a route is correct for a given request. Panics if not.
    pub fn validate_route(&self, request: &RouteRequest, route: &Route) {
        // check if route start and end is correct
        assert_eq!(route.nodes.first().unwrap(), &request.source);
        assert_eq!(route.nodes.last().unwrap(), &request.target);

        // check if there is an edge between consecutive route nodes
        let mut edges = Vec::new();
        for node_pair in route.nodes.windows(2) {
            if let [from, to] = node_pair {
                let min_edge = self
                    .out_edges
                    .edges(*from)
                    .iter()
                    .filter(|edge| edge.head == *to)
                    .min_by_key(|edge| edge.cost)
                    .expect(format!("no edge between {} and {} found", from, to).as_str());
                edges.push(min_edge);
            } else {
                panic!("Can't unpack node_pair: {:?}", node_pair);
            }
        }

        // check if cost of route is correct
        let true_cost = edges.iter().map(|edge| edge.cost).sum::<u32>();
        assert_eq!(route.cost, true_cost);
    }
}
