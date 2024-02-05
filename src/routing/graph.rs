use std::{collections::HashSet, usize};

use serde_derive::{Deserialize, Serialize};

use super::{
    edge::{DirectedHeadlessWeightedEdge, DirectedTaillessWeightedEdge, DirectedWeightedEdge},
    naive_graph::NaiveGraph,
    route::{Route, RouteRequest},
    types::VertexId,
};

/// Represents a directed graph where each node's incoming and outgoing edges are easily accessible.
///
/// This struct is designed to facilitate easy access to the neighborhood of nodes in a graph.
/// It stores edges in two vectors: `in_edges` and `out_edges`, representing incoming and outgoing edges, respectively.
///
/// For each directed edge `v -> w`:
/// - It is stored in `out_edges[v]`, allowing quick access to all successors of `v` (nodes that `v` points to).
/// - It is also stored in `in_edges[w]`, enabling efficient access to all predecessors of `w` (nodes that point to `w`).
///
#[derive(Clone, Serialize, Deserialize)]
pub struct Graph {
    pub out_edges: Vec<Vec<DirectedTaillessWeightedEdge>>,
    pub in_edges: Vec<Vec<DirectedHeadlessWeightedEdge>>,
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph {
    fn new() -> Self {
        Graph {
            out_edges: Vec::new(),
            in_edges: Vec::new(),
        }
    }

    pub fn from_naive_graph(naive_graph: &NaiveGraph) -> Graph {
        let mut graph = Graph::new();
        naive_graph.edges.iter().for_each(|edge| {
            graph.add_edge(edge);
        });

        graph
    }

    /// Retrieves the set of vertices reachable from and leading to `vertex` within a specified number of `hops`.
    /// This includes the `vertex` itself.
    ///
    /// The function explores the graph in both directions - following outgoing edges (successors) and incoming edges (predecessors).
    /// It does this iteratively for the number of hops specified, aggregating all the vertices encountered in this process.
    ///
    /// # Arguments
    /// * `vertex`: The starting vertex from which the neighborhood is calculated.
    /// * `hops`: The number of hops within which vertices are considered part of the neighborhood.
    ///
    /// # Returns
    /// A `HashSet<u32>` containing all vertices that are within `hops` hops from or to the `vertex`.
    pub fn get_neighborhood(&self, vertex: u32, hops: u32) -> HashSet<u32> {
        let mut neighbors = HashSet::new();
        neighbors.insert(vertex);

        for _ in 0..hops {
            let mut new_neighbors = HashSet::new();
            for &node in neighbors.iter() {
                new_neighbors.extend(self.out_edges[node as usize].iter().map(|edge| edge.head));
                new_neighbors.extend(self.in_edges[node as usize].iter().map(|edge| edge.tail));
            }
            neighbors.extend(new_neighbors);
        }

        neighbors
    }

    /// Adds an edge to the graph.
    pub fn add_edge(&mut self, edge: &DirectedWeightedEdge) {
        if (self.out_edges.len() as u32) <= edge.tail {
            self.out_edges.resize((edge.tail + 1) as usize, Vec::new());
        }
        self.out_edges[edge.tail as usize].push(edge.tailless());

        if (self.in_edges.len() as u32) <= edge.head {
            self.in_edges.resize((edge.head + 1) as usize, Vec::new());
        }
        self.in_edges[edge.head as usize].push(edge.headless());
    }

    /// Removes an edge from the graph.
    pub fn remove_edge(&mut self, edge: &DirectedWeightedEdge) {
        if let Some(out_edges) = self.out_edges.get_mut(edge.tail as usize) {
            out_edges.retain(|out_edge| out_edge.head != edge.head);
        }

        if let Some(in_edges) = self.in_edges.get_mut(edge.head as usize) {
            in_edges.retain(|in_edge| in_edge.tail != edge.tail);
        }
    }

    /// Removes the node from the graph.
    pub fn remove_vertex(&mut self, vertex: VertexId) {
        let out_edges = std::mem::take(&mut self.out_edges[vertex as usize]);
        out_edges.iter().for_each(|out_edge| {
            self.in_edges[out_edge.head as usize].retain(|in_edge| in_edge.tail != vertex)
        });

        let in_edges = std::mem::take(&mut self.in_edges[vertex as usize]);
        in_edges.iter().for_each(|in_edge| {
            self.out_edges[in_edge.tail as usize].retain(|out_edge| out_edge.head != vertex);
        });
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
                let min_edge = self.out_edges[*from as usize]
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
