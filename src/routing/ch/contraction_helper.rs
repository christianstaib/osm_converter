use std::collections::{BinaryHeap, HashMap};

use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::routing::graph::{DirectedEdge, Graph};

use super::binary_heap::MinimumItem;

pub struct ContractionHelper<'a> {
    graph: &'a Graph,
}

impl<'a> ContractionHelper<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Self { graph }
    }

    /// Generates shortcuts for a node v.
    ///
    /// A shortcut (u, w) is generated if ((u, v), (v, w)) is the only shortest path between u and
    /// w.
    ///
    /// Returns a vector of (Edge, Vec<Edge>) where the first entry is the shortcut and the second
    /// entry the edges the shortcut replaces.
    pub fn generate_shortcuts(
        &self,
        v: u32,
        max_hops_in_witness_search: u32,
    ) -> Vec<(DirectedEdge, Vec<DirectedEdge>)> {
        let uv_edges = &self.graph.out_edges[v as usize];
        let vw_edges = &self.graph.in_edges[v as usize];
        let max_vw_cost = vw_edges.iter().map(|edge| edge.cost).max().unwrap_or(0);

        uv_edges
            .iter()
            .par_bridge()
            .flat_map(|uv_edge| {
                let mut shortcuts = Vec::new();
                let u = uv_edge.head;
                let uv_cost = uv_edge.cost;

                let max_cost = uv_cost + max_vw_cost;
                let witness_cost = self.witness_search(u, v, max_cost, max_hops_in_witness_search);

                for vw_ede in vw_edges.iter() {
                    let w = vw_ede.tail;
                    let vw_cost = vw_ede.cost;
                    let uw_cost = uv_cost + vw_cost;
                    if &uw_cost < witness_cost.get(&w).unwrap_or(&u32::MAX) {
                        let shortcut = DirectedEdge {
                            head: u,
                            tail: w,
                            cost: uw_cost,
                        };
                        shortcuts.push((shortcut, vec![uv_edge.clone(), vw_ede.clone()]));
                    }
                }
                shortcuts
            })
            .collect()
    }

    /// Performs a forward search from `source` node.
    ///
    /// Returns a `HashMap` where each entry consists of a node identifier (u32) and the associated cost (u32) to reach that node from the `source`.
    ///
    /// Parameters:
    /// - `source`: The starting node for the search.
    /// - `without`: A node identifier to be excluded from the search. The search will ignore paths through this node.
    /// - `max_cost`: The maximum allowable cost. Nodes that can only be reached with a cost higher than this value will not be included in the results.
    /// - `max_hops`: The maximum number of hops (edges traversed) allowed. Nodes that require more hops to reach than this limit will not be included in the results.
    ///
    /// Note: The search algorithm takes into account the cost and number of hops to reach each node. Nodes are included in the resulting map only if they meet the specified conditions regarding cost and hop count, and are not the `without` node.
    pub fn witness_search(
        &self,
        source: u32,
        without: u32,
        max_cost: u32,
        max_hops: u32,
    ) -> HashMap<u32, u32> {
        let mut queue = BinaryHeap::new();
        let mut cost = HashMap::new();
        let mut hops = HashMap::new();

        queue.push(MinimumItem {
            cost: 0,
            node: source,
        });
        cost.insert(source, 0);
        hops.insert(source, 0);

        while let Some(MinimumItem { node, .. }) = queue.pop() {
            for edge in &self.graph.in_edges[node as usize] {
                let alternative_cost = cost[&node] + edge.cost;
                let new_hops = hops[&node] + 1;
                if (edge.tail != without)
                    && (alternative_cost <= max_cost)
                    && (new_hops <= max_hops)
                {
                    let current_cost = *cost.get(&edge.tail).unwrap_or(&u32::MAX);
                    if alternative_cost < current_cost {
                        queue.push(MinimumItem {
                            cost: alternative_cost,
                            node: edge.tail,
                        });
                        cost.insert(edge.tail, alternative_cost);
                        hops.insert(edge.tail, new_hops);
                    }
                }
            }
        }

        cost
    }
}
