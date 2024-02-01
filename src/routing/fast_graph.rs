use super::{
    fast_edge_access::FastEdgeAccess,
    graph::Graph,
    naive_graph::NaiveGraph,
    route::{Route, RouteRequest},
};

#[derive(Clone, Debug)]
pub struct FastEdge {
    pub target: u32,
    pub cost: u32,
}

#[derive(Clone)]
/// Gives fast access to predecessor and successor in a graph.
/// As FastGraph uses FastEdgeAccess, an out_edges head is acutally its tail.
pub struct FastGraph {
    pub num_nodes: u32,
    pub in_edges: FastEdgeAccess,
    pub out_edges: FastEdgeAccess,
}

impl FastGraph {
    pub fn from_graph(graph: &Graph) -> FastGraph {
        let num_nodes = graph.in_edges.len() as u32;

        let in_edges = graph.in_edges.iter().flatten().cloned().collect();
        let in_edges = FastEdgeAccess::new(&in_edges);

        let out_edges = graph
            .out_edges
            .iter()
            .flatten()
            .map(|edge| edge.get_inverted())
            .collect();
        let out_edges = FastEdgeAccess::new(&out_edges);

        FastGraph {
            num_nodes,
            in_edges,
            out_edges,
        }
    }
    pub fn outgoing_edges(&self, source: u32) -> &[FastEdge] {
        self.in_edges.outgoing_edges(source)
    }

    pub fn incoming_edges(&self, target: u32) -> &[FastEdge] {
        self.out_edges.outgoing_edges(target)
    }

    pub fn from_naive_graph(graph: &NaiveGraph) -> FastGraph {
        let graph = graph.clone();

        let forward_edges = FastEdgeAccess::new(&graph.edges);

        let inverted_edges = graph.edges.iter().map(|edge| edge.get_inverted()).collect();
        let backward_edges = FastEdgeAccess::new(&inverted_edges);

        FastGraph {
            num_nodes: graph.nodes.len() as u32,
            in_edges: forward_edges,
            out_edges: backward_edges,
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
                    .in_edges
                    .outgoing_edges(*from)
                    .iter()
                    .filter(|edge| edge.target == *to)
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
