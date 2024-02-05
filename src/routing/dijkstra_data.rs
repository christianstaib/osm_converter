use std::usize;

use super::{
    queue::heap_queue::{HeapQueue, State},
    route::Route,
    types::VertexId,
};

#[derive(Clone)]
pub struct DijsktraEntry {
    pub predecessor: VertexId,
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
    pub verticies: Vec<DijsktraEntry>,
}

impl DijkstraData {
    pub fn new(num_nodes: usize, source: VertexId) -> DijkstraData {
        let mut queue = HeapQueue::new();
        let mut nodes = vec![DijsktraEntry::new(); num_nodes];
        nodes[source as usize].cost = 0;
        queue.insert(0, source);
        DijkstraData {
            queue,
            verticies: nodes,
        }
    }

    pub fn pop(&mut self) -> Option<State> {
        while let Some(state) = self.queue.pop() {
            if !self.verticies[state.value as usize].is_expanded {
                self.verticies[state.value as usize].is_expanded = true;
                return Some(state);
            }
        }

        None
    }

    pub fn update(&mut self, source: VertexId, target: VertexId, edge_cost: u32) {
        let alternative_cost = self.verticies[source as usize].cost + edge_cost;
        let current_cost = self.verticies[target as usize].cost;
        if alternative_cost < current_cost {
            self.verticies[target as usize].predecessor = source;
            self.verticies[target as usize].cost = alternative_cost;
            self.queue.insert(alternative_cost, target);
        }
    }

    pub fn get_route(&self, target: VertexId) -> Option<Route> {
        if self.verticies[target as usize].cost != u32::MAX {
            let mut route = vec![target];
            let mut current = target;
            while self.verticies[current as usize].predecessor != u32::MAX {
                current = self.verticies[current as usize].predecessor;
                route.push(current);
            }
            route.reverse();
            return Some(Route {
                cost: self.verticies[target as usize].cost,
                nodes: route,
            });
        }
        None
    }

    pub fn get_scanned_points(&self) -> Vec<usize> {
        self.verticies
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.cost != u32::MAX)
            .map(|(i, _)| i)
            .collect()
    }
}
