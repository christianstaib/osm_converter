use super::{
    graph::{Edge, Graph},
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
pub struct FastGraph {
    // pub nodes: Vec<Point>,
    pub num_nodes: u32,
    pub forward_edges: FastEdgeAccess,
    pub backward_edges: FastEdgeAccess,
}

#[derive(Clone)]
pub struct FastEdgeAccess {
    pub edges: Vec<FastEdge>,
    pub edges_start_at: Vec<u32>,
}

impl FastEdgeAccess {
    pub fn new(edges: &Vec<Edge>) -> FastEdgeAccess {
        let mut edges = edges.clone();

        let mut edges_start_at: Vec<u32> = vec![0; edges.len() + 1];

        // temporarrly adding a node in order to generate the list
        edges.push(Edge {
            source: edges.len() as u32,
            target: 0,
            cost: 0,
        });
        edges.sort_unstable_by_key(|edge| edge.source);

        let mut current = 0;
        for (i, edge) in edges.iter().enumerate() {
            if edge.source != current {
                for index in (current + 1)..=edge.source {
                    edges_start_at[index as usize] = i as u32;
                }
                current = edge.source;
            }
        }
        edges.pop();
        let edges: Vec<_> = edges.iter().map(|edge| edge.make_fast()).collect();
        let edges_start_at = edges_start_at.clone();

        FastEdgeAccess {
            edges,
            edges_start_at,
        }
    }

    pub fn outgoing_edges(&self, source: u32) -> &[FastEdge] {
        let start = self.edges_start_at[source as usize] as usize;
        let end = self.edges_start_at[source as usize + 1] as usize;

        &self.edges[start..end]
    }
}

impl FastGraph {
    pub fn from_graph(graph: &Graph) -> FastGraph {
        let num_nodes = graph.forward_edges.len() as u32;

        let forward_edges = graph.forward_edges.iter().flatten().cloned().collect();
        let forward_edges = FastEdgeAccess::new(&forward_edges);

        let backward_edges = graph
            .backward_edges
            .iter()
            .flatten()
            .map(|edge| edge.get_inverted())
            .collect();
        let backward_edges = FastEdgeAccess::new(&backward_edges);

        FastGraph {
            num_nodes,
            forward_edges,
            backward_edges,
        }
    }
    pub fn outgoing_edges(&self, source: u32) -> &[FastEdge] {
        self.forward_edges.outgoing_edges(source)
    }

    pub fn incoming_edges(&self, target: u32) -> &[FastEdge] {
        self.backward_edges.outgoing_edges(target)
    }

    pub fn new(graph: &NaiveGraph) -> FastGraph {
        let graph = graph.clone();

        let forward_edges = FastEdgeAccess::new(&graph.edges);

        let inverted_edges = graph.edges.iter().map(|edge| edge.get_inverted()).collect();
        let backward_edges = FastEdgeAccess::new(&inverted_edges);

        FastGraph {
            num_nodes: graph.nodes.len() as u32,
            forward_edges,
            backward_edges,
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
                    .forward_edges
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
