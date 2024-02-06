use std::{
    collections::BinaryHeap,
    time::{Duration, Instant},
};

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use indicatif::ProgressIterator;

use crate::routing::{
    ch::contractor::ContractedGraph,
    edge::DirectedEdge,
    fast_graph::FastGraph,
    hl::{hub_graph::HubGraph, label::Label, label_entry::LabelEntry},
    path::{Path, PathRequest},
    queue::heap_queue::State,
    types::VertexId,
};

#[derive(Clone)]
pub struct ChDijkstra {
    pub graph: FastGraph,
    pub shortcuts: HashMap<DirectedEdge, u32>,
    pub levels: Vec<Vec<VertexId>>,
}

impl ChDijkstra {
    pub fn new(contracted_grap: &ContractedGraph) -> ChDijkstra {
        let shortcuts = contracted_grap.shortcuts_map.iter().cloned().collect();
        let graph = FastGraph::from_graph(&contracted_grap.graph);
        ChDijkstra {
            graph,
            shortcuts,
            levels: contracted_grap.levels.clone(),
        }
    }

    pub fn get_hl(&self) -> HubGraph {
        println!("fast hl generation");

        let mut out_labels: Vec<_> = (0..self.graph.num_nodes())
            .map(|vertex| {
                let entry = LabelEntry {
                    id: vertex,
                    cost: 0,
                    predecessor: vertex,
                };

                Label {
                    entries: vec![entry],
                }
            })
            .collect();

        let mut in_labels = out_labels.clone();

        let mut merge_time = 0;
        let mut sort_time = 0;
        let mut prune_time = 0;

        let start_all = Instant::now();
        for (i, level_list) in self.levels.iter().rev().enumerate().progress() {
            for vertex in level_list {
                let start = Instant::now();
                for out_edge in self.graph.out_edges(*vertex) {
                    let mut head_label_entries = out_labels[out_edge.head as usize].entries.clone();
                    head_label_entries.iter_mut().for_each(|entry| {
                        if entry.id == out_edge.head {
                            entry.predecessor = *vertex;
                        }
                        entry.cost += out_edge.cost
                    });

                    out_labels[*vertex as usize]
                        .entries
                        .extend(head_label_entries);
                }
                merge_time += start.elapsed().as_micros();

                let start = Instant::now();
                out_labels[*vertex as usize].sort_and_clean();
                sort_time += start.elapsed().as_micros();

                let start = Instant::now();
                out_labels[*vertex as usize].prune_forward(&in_labels);
                prune_time += start.elapsed().as_micros();

                if i % 10000 == 0 {
                    println!("i: {}", i);
                    println!(
                        "it/s: {:>9.2}",
                        i as f32 / start_all.elapsed().as_secs_f32()
                    );
                    let all_micros = start_all.elapsed().as_micros() as f64 / 100.0;
                    println!("merge time: {:>2.2}", merge_time as f64 / all_micros);
                    println!("sort time:  {:>2.2}", sort_time as f64 / all_micros);
                    println!("prune time: {:>2.2}", prune_time as f64 / all_micros);
                    println!("");
                }

                for in_edge in self.graph.in_edges(*vertex) {
                    let mut tail_label_entries = in_labels[in_edge.tail as usize].entries.clone();
                    tail_label_entries.iter_mut().for_each(|entry| {
                        if entry.id == in_edge.tail {
                            entry.predecessor = *vertex;
                        }
                        entry.cost += in_edge.cost
                    });

                    in_labels[*vertex as usize]
                        .entries
                        .extend(tail_label_entries);
                }
                in_labels[*vertex as usize].sort_and_clean();
                in_labels[*vertex as usize].prune_backward(&out_labels);
            }
        }

        HubGraph {
            forward_labels: out_labels,
            backward_labels: in_labels,
            shortcuts: self.shortcuts.clone(),
        }
    }

    ///
    /// (contact_node, cost)
    pub fn get_forward_label(&self, source: u32, depth_limit: u32) -> HashMap<u32, (u32, u32)> {
        let mut costs = HashMap::new();
        let mut predecessor = HashMap::new();
        let mut open = BinaryHeap::new();
        let mut expanded = HashSet::new();

        open.push(State {
            key: 0,
            value: source,
        });
        costs.insert(source, 0);
        predecessor.insert(source, source);

        while let Some(state) = open.pop() {
            let current_node = state.value;
            if !expanded.contains(&current_node) {
                expanded.insert(current_node);

                let current_node_cost = *costs.get(&current_node).unwrap();

                let backward_search = self.backward_search(current_node, depth_limit);
                let incoming_min = backward_search
                    .iter()
                    .filter_map(|(node, cost)| Some(costs.get(node)? + *cost))
                    .min()
                    .unwrap_or(u32::MAX);

                if current_node_cost > incoming_min {
                    costs.remove(&current_node);
                    continue;
                }

                self.graph.out_edges(state.value).iter().for_each(|edge| {
                    let alternative_cost = current_node_cost + edge.cost;
                    let current_cost = *costs.get(&edge.head).unwrap_or(&u32::MAX);
                    if alternative_cost < current_cost {
                        costs.insert(edge.head, alternative_cost);
                        predecessor.insert(edge.head, current_node);
                        open.push(State {
                            key: alternative_cost,
                            value: edge.head,
                        });
                    }
                });
            }
        }

        costs
            .iter()
            .map(|(id, cost)| (*id, (*cost, *predecessor.get(id).unwrap())))
            .collect()
    }

    ///
    /// (contact_node, cost)
    pub fn get_backward_label(&self, source: u32, depth_limit: u32) -> HashMap<u32, (u32, u32)> {
        let mut costs = HashMap::new();
        let mut predecessor = HashMap::new();
        let mut open = BinaryHeap::new();
        let mut expanded = HashSet::new();

        open.push(State {
            key: 0,
            value: source,
        });
        costs.insert(source, 0);
        predecessor.insert(source, source);

        while let Some(state) = open.pop() {
            let current_node = state.value;
            if !expanded.contains(&current_node) {
                expanded.insert(current_node);

                let current_node_cost = *costs.get(&current_node).unwrap();

                // TODO this is not working, but get_forward_label is working.
                let backward_search = self.forward_search(current_node, depth_limit);
                let incoming_min = backward_search
                    .iter()
                    .map(|(node, cost)| {
                        costs
                            .get(node)
                            .unwrap_or(&u32::MAX)
                            .checked_add(*cost)
                            .unwrap_or(u32::MAX)
                    })
                    .min()
                    .unwrap_or(u32::MAX);

                if current_node_cost > incoming_min {
                    costs.remove(&current_node);
                    continue;
                }
                //

                self.graph.in_edges(state.value).iter().for_each(|edge| {
                    let alternative_cost = current_node_cost + edge.cost;
                    let current_cost = *costs.get(&edge.tail).unwrap_or(&u32::MAX);
                    if alternative_cost < current_cost {
                        costs.insert(edge.tail, alternative_cost);
                        predecessor.insert(edge.tail, current_node);
                        open.push(State {
                            key: alternative_cost,
                            value: edge.tail,
                        });
                    }
                });
            }
        }

        costs
            .iter()
            .map(|(id, cost)| (*id, (*cost, *predecessor.get(id).unwrap())))
            .collect()
    }

    ///
    /// (contact_node, cost)
    pub fn forward_search(&self, target: u32, depth_limit: u32) -> HashMap<u32, u32> {
        let mut costs = HashMap::new();
        let mut depth = HashMap::new();
        let mut open = BinaryHeap::new();
        let mut expanded = HashSet::new();

        open.push(State {
            key: 0,
            value: target,
        });
        costs.insert(target, 0);
        depth.insert(target, 0);

        while let Some(state) = open.pop() {
            let current_node = state.value;
            let current_node_cost = *costs.get(&current_node).unwrap();
            let new_depth = depth.get(&current_node).unwrap() + 1;
            if !expanded.contains(&current_node) && (new_depth <= depth_limit) {
                expanded.insert(current_node);

                self.graph.out_edges(state.value).iter().for_each(|edge| {
                    let alternative_cost = current_node_cost + edge.cost;
                    let current_cost = *costs.get(&edge.head).unwrap_or(&u32::MAX);
                    if alternative_cost < current_cost {
                        costs.insert(edge.head, alternative_cost);
                        depth.insert(edge.head, new_depth);
                        open.push(State {
                            key: alternative_cost,
                            value: edge.head,
                        });
                    }
                });
            }
        }
        costs
    }

    ///
    /// (contact_node, cost)
    pub fn backward_search(&self, target: u32, depth_limit: u32) -> HashMap<u32, u32> {
        let mut costs = HashMap::new();
        let mut depth = HashMap::new();
        let mut open = BinaryHeap::new();
        let mut expanded = HashSet::new();

        open.push(State {
            key: 0,
            value: target,
        });
        costs.insert(target, 0);
        depth.insert(target, 0);

        while let Some(state) = open.pop() {
            let current_node = state.value;
            let current_node_cost = *costs.get(&current_node).unwrap();
            let new_depth = depth.get(&current_node).unwrap() + 1;
            if !expanded.contains(&current_node) && (new_depth <= depth_limit) {
                expanded.insert(current_node);

                self.graph.in_edges(state.value).iter().for_each(|edge| {
                    let alternative_cost = current_node_cost + edge.cost;
                    let current_cost = *costs.get(&edge.tail).unwrap_or(&u32::MAX);
                    if alternative_cost < current_cost {
                        costs.insert(edge.tail, alternative_cost);
                        depth.insert(edge.tail, new_depth);
                        open.push(State {
                            key: alternative_cost,
                            value: edge.tail,
                        });
                    }
                });
            }
        }
        costs
    }

    /// (contact_node, cost)
    pub fn get_cost(&self, request: &PathRequest) -> Option<u32> {
        let mut forward_costs = HashMap::new();
        let mut backward_costs = HashMap::new();

        let mut forward_open = BinaryHeap::new();
        let mut backward_open = BinaryHeap::new();

        let mut forward_expanded = HashSet::new();
        let mut backward_expaned = HashSet::new();

        forward_open.push(State {
            key: 0,
            value: request.source,
        });
        forward_costs.insert(request.source, 0);

        backward_open.push(State {
            key: 0,
            value: request.target,
        });
        backward_costs.insert(request.target, 0);

        let mut minimal_cost = u32::MAX;

        while !forward_open.is_empty() || !backward_open.is_empty() {
            if let Some(forward_state) = forward_open.pop() {
                let current_node = forward_state.value;
                if !forward_expanded.contains(&current_node) {
                    forward_expanded.insert(current_node);

                    if backward_expaned.contains(&current_node) {
                        let contact_cost = forward_costs.get(&current_node).unwrap()
                            + backward_costs.get(&current_node).unwrap();
                        if contact_cost < minimal_cost {
                            minimal_cost = contact_cost;
                        }
                    }

                    self.graph
                        .out_edges(forward_state.value)
                        .iter()
                        .for_each(|edge| {
                            let alternative_cost =
                                forward_costs.get(&current_node).unwrap() + edge.cost;
                            let current_cost = *forward_costs.get(&edge.head).unwrap_or(&u32::MAX);
                            if alternative_cost < current_cost {
                                forward_costs.insert(edge.head, alternative_cost);
                                forward_open.push(State {
                                    key: alternative_cost,
                                    value: edge.head,
                                });
                            }
                        });
                }
            }

            if let Some(backward_state) = backward_open.pop() {
                let current_node = backward_state.value;
                if !backward_expaned.contains(&current_node) {
                    backward_expaned.insert(current_node);

                    if forward_expanded.contains(&current_node) {
                        let contact_cost = forward_costs.get(&current_node).unwrap()
                            + backward_costs.get(&current_node).unwrap();
                        if contact_cost < minimal_cost {
                            minimal_cost = contact_cost;
                        }
                    }

                    self.graph
                        .in_edges(backward_state.value)
                        .iter()
                        .for_each(|edge| {
                            let alternative_cost =
                                backward_costs.get(&current_node).unwrap() + edge.cost;
                            let current_cost = *backward_costs.get(&edge.tail).unwrap_or(&u32::MAX);
                            if alternative_cost < current_cost {
                                backward_costs.insert(edge.tail, alternative_cost);
                                backward_open.push(State {
                                    key: alternative_cost,
                                    value: edge.tail,
                                });
                            }
                        });
                }
            }
        }

        if minimal_cost != u32::MAX {
            return Some(minimal_cost);
        }
        None
    }
    /// (contact_node, cost)
    pub fn get_route(&self, request: &PathRequest) -> Option<Path> {
        let mut forward_costs = HashMap::new();
        let mut backward_costs = HashMap::new();

        let mut forward_predecessor = HashMap::new();
        let mut backward_predecessor = HashMap::new();

        let mut forward_open = BinaryHeap::new();
        let mut backward_open = BinaryHeap::new();

        let mut forward_expanded = HashSet::new();
        let mut backward_expaned = HashSet::new();

        forward_open.push(State {
            key: 0,
            value: request.source,
        });
        forward_costs.insert(request.source, 0);

        backward_open.push(State {
            key: 0,
            value: request.target,
        });
        backward_costs.insert(request.target, 0);

        let mut minimal_cost = u32::MAX;
        let mut meeting_node = u32::MAX;

        while !forward_open.is_empty() || !backward_open.is_empty() {
            if let Some(forward_state) = forward_open.pop() {
                let current_node = forward_state.value;
                if !forward_expanded.contains(&current_node) {
                    forward_expanded.insert(current_node);

                    if backward_expaned.contains(&current_node) {
                        let contact_cost = forward_costs.get(&current_node).unwrap()
                            + backward_costs.get(&current_node).unwrap();
                        if contact_cost < minimal_cost {
                            minimal_cost = contact_cost;
                            meeting_node = forward_state.value;
                        }
                    }

                    self.graph
                        .out_edges(forward_state.value)
                        .iter()
                        .for_each(|edge| {
                            let alternative_cost =
                                forward_costs.get(&current_node).unwrap() + edge.cost;
                            let current_cost = *forward_costs.get(&edge.head).unwrap_or(&u32::MAX);
                            if alternative_cost < current_cost {
                                forward_costs.insert(edge.head, alternative_cost);
                                forward_predecessor.insert(edge.head, current_node);
                                forward_open.push(State {
                                    key: alternative_cost,
                                    value: edge.head,
                                });
                            }
                        });
                }
            }

            if let Some(backward_state) = backward_open.pop() {
                let current_node = backward_state.value;
                if !backward_expaned.contains(&current_node) {
                    backward_expaned.insert(current_node);

                    if forward_expanded.contains(&current_node) {
                        let contact_cost = forward_costs.get(&current_node).unwrap()
                            + backward_costs.get(&current_node).unwrap();
                        if contact_cost < minimal_cost {
                            minimal_cost = contact_cost;
                            meeting_node = backward_state.value;
                        }
                    }

                    self.graph
                        .in_edges(backward_state.value)
                        .iter()
                        .for_each(|edge| {
                            let alternative_cost =
                                backward_costs.get(&current_node).unwrap() + edge.cost;
                            let current_cost = *backward_costs.get(&edge.tail).unwrap_or(&u32::MAX);
                            if alternative_cost < current_cost {
                                backward_costs.insert(edge.tail, alternative_cost);
                                backward_predecessor.insert(edge.tail, current_node);
                                backward_open.push(State {
                                    key: alternative_cost,
                                    value: edge.tail,
                                });
                            }
                        });
                }
            }
        }

        get_route(
            meeting_node,
            minimal_cost,
            forward_predecessor,
            backward_predecessor,
        )
    }
}

fn get_route(
    meeting_node: u32,
    meeting_cost: u32,
    forward_predecessor: HashMap<u32, u32>,
    backward_predecessor: HashMap<u32, u32>,
) -> Option<Path> {
    if meeting_cost == u32::MAX {
        return None;
    }
    let mut route = Vec::new();
    let mut current = meeting_node;
    route.push(current);
    while let Some(new_current) = forward_predecessor.get(&current) {
        current = *new_current;
    }
    current = meeting_node;
    while let Some(new_current) = backward_predecessor.get(&current) {
        current = *new_current;
    }
    let route = Path {
        verticies: route,
        cost: meeting_cost,
    };
    Some(route)
}
