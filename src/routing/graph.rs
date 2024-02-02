use std::{collections::HashSet, usize};

use serde_derive::{Deserialize, Serialize};

use super::{fast_graph::FastOutEdge, naive_graph::NaiveGraph};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, Debug)]
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

    pub fn get_fast_edge(&self) -> FastOutEdge {
        FastOutEdge {
            head: self.tail,
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

    pub fn validate(&self) {
        // assert all edges are sorted in corretly
        for (tail, out_edges) in self.out_edges.iter().enumerate() {
            let tail = tail as u32;
            for out_edge in out_edges.iter() {
                assert_eq!(tail, out_edge.tail);
            }
        }
        for (head, in_edges) in self.in_edges.iter().enumerate() {
            let head = head as u32;
            for in_edge in in_edges.iter() {
                assert_eq!(head, in_edge.head);
            }
        }

        // assert in_edges and out_edges contain the same edges
        let mut out_edges: Vec<_> = self.out_edges.iter().flatten().cloned().collect();
        out_edges.sort();
        let mut in_edges: Vec<_> = self.in_edges.iter().flatten().cloned().collect();
        in_edges.sort();

        assert_eq!(out_edges.len(), in_edges.len());
        for i in 0..out_edges.len() {
            assert_eq!(out_edges[i], in_edges[i]);
        }
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
