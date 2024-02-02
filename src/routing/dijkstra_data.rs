use std::usize;

use super::{
    fast_graph::FastOutEdge,
    queue::heap_queue::{HeapQueue, State},
    route::Route,
};

#[derive(Clone)]
pub struct DijsktraEntry {
    pub predecessor: u32,
    pub cost: u32,
    pub is_expanded: bool,
}

impl DijsktraEntry {
    fn new() -> DijsktraEntry {
        DijsktraEntry {
            predecessor: u32::MAX,
            cost: u32::MAX,
            is_expanded: false,
        }
    }
}

#[derive(Clone)]
pub struct DijkstraData {
    pub queue: HeapQueue,
    pub nodes: Vec<DijsktraEntry>,
}

impl DijkstraData {
    pub fn new(num_nodes: usize, source: u32) -> DijkstraData {
        let mut queue = HeapQueue::new();
        let mut nodes = vec![DijsktraEntry::new(); num_nodes];
        nodes[source as usize].cost = 0;
        queue.insert(0, source);
        DijkstraData { queue, nodes }
    }

    pub fn pop(&mut self) -> Option<State> {
        while let Some(state) = self.queue.pop() {
            if !self.nodes[state.value as usize].is_expanded {
                self.nodes[state.value as usize].is_expanded = true;
                return Some(state);
            }
        }

        None
    }

    pub fn update(&mut self, source: u32, edge: &FastOutEdge) {
        let alternative_cost = self.nodes[source as usize].cost + edge.cost;
        let current_cost = self.nodes[edge.head as usize].cost;
        if alternative_cost < current_cost {
            self.nodes[edge.head as usize].predecessor = source;
            self.nodes[edge.head as usize].cost = alternative_cost;
            self.queue.insert(alternative_cost, edge.head);
        }
    }

    pub fn get_route(&self, target: u32) -> Option<Route> {
        if self.nodes[target as usize].cost != u32::MAX {
            let mut route = vec![target];
            let mut current = target;
            while self.nodes[current as usize].predecessor != u32::MAX {
                current = self.nodes[current as usize].predecessor;
                route.insert(0, current);
            }
            return Some(Route {
                cost: self.nodes[target as usize].cost,
                nodes: route,
            });
        }
        None
    }

    pub fn get_scanned_points(&self) -> Vec<usize> {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.cost != u32::MAX)
            .map(|(i, _)| i)
            .collect()
    }
}
