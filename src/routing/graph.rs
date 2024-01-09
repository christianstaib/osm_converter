use std::{collections::HashSet, usize};

use serde_derive::{Deserialize, Serialize};

use super::{fast_graph::FastEdge, naive_graph::NaiveGraph};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    pub source: u32,
    pub target: u32,
    pub cost: u32,
}

impl Edge {
    pub fn new(source: u32, target: u32, cost: u32) -> Edge {
        Edge {
            source,
            target,
            cost,
        }
    }

    pub fn get_inverted(&self) -> Edge {
        Edge {
            source: self.target,
            target: self.source,
            cost: self.cost,
        }
    }
}

impl Edge {
    pub fn make_fast(&self) -> FastEdge {
        FastEdge {
            target: self.target,
            cost: self.cost,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Graph {
    //pub nodes: Vec<Point>,
    pub forward_edges: Vec<Vec<Edge>>,
    pub backward_edges: Vec<Vec<Edge>>,
}

impl Graph {
    pub fn from_naive_graph(graph: &NaiveGraph) -> Graph {
        let max_node_id = graph
            .edges
            .iter()
            .map(|edge| std::cmp::max(edge.source, edge.target))
            .max()
            .unwrap_or(0);
        let mut forward_edges = vec![Vec::new(); (max_node_id + 1) as usize];
        let mut backward_edges = vec![Vec::new(); (max_node_id + 1) as usize];
        graph.edges.iter().for_each(|edge| {
            forward_edges[edge.source as usize].push(edge.clone());
            backward_edges[edge.target as usize].push(edge.clone());
        });

        Graph {
            // nodes: graph.nodes.clone(),
            forward_edges,
            backward_edges,
        }
    }

    pub fn get_neighborhood(&self, node: u32, hops: u32) -> HashSet<u32> {
        let mut neighbors = HashSet::new();
        neighbors.insert(node);

        for _ in 0..hops {
            let mut new_neighsbors = HashSet::new();
            for &node in neighbors.iter() {
                new_neighsbors.extend(
                    self.forward_edges[node as usize]
                        .iter()
                        .map(|edge| edge.target),
                );
                new_neighsbors.extend(
                    self.backward_edges[node as usize]
                        .iter()
                        .map(|edge| edge.source),
                );
            }
            neighbors.extend(new_neighsbors);
        }

        neighbors
    }

    /// Adds an edge to the graph.
    pub fn add_edge(&mut self, edge: &Edge) {
        self.forward_edges[edge.source as usize].push(edge.clone());
        self.backward_edges[edge.target as usize].push(edge.clone());
    }

    /// Removes the node from the graph.
    ///
    /// Removing means, that afterwards, there will be no edges going into node or going out of
    /// node.
    pub fn disconnect(&mut self, node: u32) {
        let outgoing_edges = std::mem::take(&mut self.forward_edges[node as usize]);
        outgoing_edges.iter().for_each(|outgoing_edge| {
            let idx = self.backward_edges[outgoing_edge.target as usize]
                .iter()
                .position(|backward_edge| outgoing_edge == backward_edge)
                .unwrap();
            self.backward_edges[outgoing_edge.target as usize].remove(idx);
        });

        let incoming_edges = std::mem::take(&mut self.backward_edges[node as usize]);
        incoming_edges.iter().for_each(|incoming_edge| {
            let idx = self.forward_edges[incoming_edge.source as usize]
                .iter()
                .position(|forward_edge| forward_edge == incoming_edge)
                .unwrap();
            self.forward_edges[incoming_edge.source as usize].remove(idx);
        });
    }
}
