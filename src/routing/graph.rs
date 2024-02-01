use std::{collections::HashSet, usize};

use serde_derive::{Deserialize, Serialize};

use super::{fast_graph::FastEdge, naive_graph::NaiveGraph};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectedEdge {
    pub head: u32,
    pub tail: u32,
    pub cost: u32,
}

impl DirectedEdge {
    pub fn new(head: u32, tail: u32, cost: u32) -> DirectedEdge {
        DirectedEdge { head, tail, cost }
    }

    pub fn get_inverted(&self) -> DirectedEdge {
        DirectedEdge {
            head: self.tail,
            tail: self.head,
            cost: self.cost,
        }
    }

    pub fn get_fast_edge(&self) -> FastEdge {
        FastEdge {
            target: self.tail,
            cost: self.cost,
        }
    }
}

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
    pub in_edges: Vec<Vec<DirectedEdge>>,
    pub out_edges: Vec<Vec<DirectedEdge>>,
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph {
    fn new() -> Self {
        Graph {
            in_edges: Vec::new(),
            out_edges: Vec::new(),
        }
    }

    pub fn from_naive_graph(naive_graph: &NaiveGraph) -> Graph {
        let mut graph = Graph::new();
        naive_graph.edges.iter().for_each(|edge| {
            graph.add_edge(edge);
        });

        graph
    }

    pub fn in_edges(&self, vertex: u32) -> &[DirectedEdge] {
        self.in_edges.get(vertex as usize).unwrap()
    }

    pub fn out_edges(&self, vertex: u32) -> &[DirectedEdge] {
        self.out_edges.get(vertex as usize).unwrap()
    }

    /// Returns all verticies reachable within `hops` hops of `vertex` as well all verticies that can
    /// reach `vertex` within `hops` hops. This includes `vertex` itself.
    pub fn get_neighborhood(&self, vertex: u32, hops: u32) -> HashSet<u32> {
        let mut neighbors = HashSet::new();
        neighbors.insert(vertex);

        for _ in 0..hops {
            let mut new_neighbors = HashSet::new();
            for &node in neighbors.iter() {
                new_neighbors.extend(self.in_edges[node as usize].iter().map(|edge| edge.tail));
                new_neighbors.extend(self.out_edges[node as usize].iter().map(|edge| edge.head));
            }
            neighbors.extend(new_neighbors);
        }

        neighbors
    }

    /// Adds an edge to the graph.
    pub fn add_edge(&mut self, edge: &DirectedEdge) {
        if (self.in_edges.len() as u32) <= edge.head {
            self.in_edges.resize((edge.head + 1) as usize, Vec::new());
        }
        self.in_edges[edge.head as usize].push(edge.clone());

        if (self.out_edges.len() as u32) <= edge.tail {
            self.out_edges.resize((edge.tail + 1) as usize, Vec::new());
        }
        self.out_edges[edge.tail as usize].push(edge.clone());
    }

    /// Removes an edge from the graph.
    pub fn remove_edge(&mut self, edge: &DirectedEdge) {
        if let Some(in_edges) = self.in_edges.get_mut(edge.head as usize) {
            in_edges.retain(|in_edge| in_edge != edge);
        }

        if let Some(out_edges) = self.out_edges.get_mut(edge.tail as usize) {
            out_edges.retain(|out_edge| out_edge != edge);
        }
    }

    /// Removes the node from the graph.
    pub fn remove_vertex(&mut self, node: u32) {
        let in_edges = std::mem::take(&mut self.in_edges[node as usize]);
        in_edges.iter().for_each(|in_edge| {
            self.out_edges[in_edge.tail as usize].retain(|out_edge| out_edge != in_edge);
        });

        let out_edges = std::mem::take(&mut self.out_edges[node as usize]);
        out_edges.iter().for_each(|out_edge| {
            self.in_edges[out_edge.head as usize].retain(|in_edge| in_edge != out_edge)
        });
    }
}
