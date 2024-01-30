use ahash::{HashMap, HashMapExt};
use indicatif::ParallelProgressIterator;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde_derive::{Deserialize, Serialize};

use crate::routing::{route::RouteRequest, simple_algorithms::ch_bi_dijkstra::ChDijkstra};

use super::label_entry::LabelEntry;

#[derive(Serialize, Deserialize)]
pub struct Label {
    pub label: Vec<LabelEntry>,
}

impl Label {
    pub fn new(map: &HashMap<u32, (u32, u32)>) -> Label {
        let mut labels: Vec<_> = map
            .iter()
            .map(|(id, (cost, predecessor))| LabelEntry {
                id: *id,
                cost: *cost,
                predecessor: *predecessor,
            })
            .collect();
        labels.sort_unstable_by_key(|entry| entry.id);
        labels.shrink_to_fit();

        Label { label: labels }
    }

    pub fn prune_forward(&mut self, backward_labels: &Vec<Label>) {
        self.label = self
            .label
            .iter()
            .filter(|entry| {
                let backward_label = backward_labels.get(entry.id as usize).unwrap();
                let true_cost = self.get_cost(backward_label).unwrap();
                entry.cost == true_cost
            })
            .cloned()
            .collect();
    }

    pub fn prune_backward(&mut self, forward_label: &Vec<Label>) {
        self.label = self
            .label
            .iter()
            .filter(|entry| {
                let forward_label = forward_label.get(entry.id as usize).unwrap();
                let true_cost = self.get_cost(forward_label).unwrap();
                entry.cost == true_cost
            })
            .cloned()
            .collect();
    }

    pub fn set_predecessor(&mut self) {
        let mut id_idx = HashMap::with_capacity(self.label.len());

        for idx in 0..self.label.len() {
            id_idx.insert(self.label[idx].id, idx as u32);
        }

        for entry in self.label.iter_mut() {
            entry.predecessor = *id_idx.get(&entry.predecessor).unwrap();
        }
    }

    // pub fn get_cost(&self, other: &Label) -> Option<u32> {
    //     let mut i_self = 0;
    //     let mut i_other = 0;

    //     let mut cost = u32::MAX;

    //     while i_self < self.label.len() && i_other < other.label.len() {
    //         let self_entry = &self.label[i_self];
    //         let other_entry = &other.label[i_other];

    //         match self_entry.id.cmp(&other_entry.id) {
    //             std::cmp::Ordering::Less => i_self += 1,
    //             std::cmp::Ordering::Equal => {
    //                 i_self += 1;
    //                 i_other += 1;

    //                 let alternative_cost = self_entry.cost + other_entry.cost;
    //                 if alternative_cost < cost {
    //                     cost = alternative_cost;
    //                 }
    //             }
    //             std::cmp::Ordering::Greater => i_other += 1,
    //         }
    //     }

    //     if cost != u32::MAX {
    //         return Some(cost);
    //     }

    //     None
    // }

    // pub fn get_route(&self, other: &Label) -> Option<(u32, Vec<u32>)> {
    //     let (cost, mut i_self, mut i_other) = self.get_idx(other)?;

    //     let mut route = Vec::new();

    //     while self.label[i_self as usize].predecessor != self.label[i_self as usize].id {
    //         route.insert(0, self.label[i_self as usize].predecessor);
    //     }

    //     Some((cost, route))
    // }

    pub fn get_cost(&self, other: &Label) -> Option<u32> {
        Some(self.get_idx(other)?.0)
    }

    /// cost, i_self, i_other
    pub fn get_idx(&self, other: &Label) -> Option<(u32, u32, u32)> {
        let mut i_self = 0;
        let mut i_other = 0;

        let mut cost = u32::MAX;

        while i_self < self.label.len() && i_other < other.label.len() {
            let self_entry = &self.label[i_self];
            let other_entry = &other.label[i_other];

            match self_entry.id.cmp(&other_entry.id) {
                std::cmp::Ordering::Less => i_self += 1,
                std::cmp::Ordering::Equal => {
                    i_self += 1;
                    i_other += 1;

                    let alternative_cost = self_entry.cost + other_entry.cost;
                    if alternative_cost < cost {
                        cost = alternative_cost;
                    }
                }
                std::cmp::Ordering::Greater => i_other += 1,
            }
        }

        if cost != u32::MAX {
            return Some((cost, i_self as u32, i_other as u32));
        }

        None
    }
}
