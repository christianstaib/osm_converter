use std::usize;

use ahash::{HashMap, HashMapExt};
use rayon::{
    iter::{IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSliceMut,
};
use serde_derive::{Deserialize, Serialize};
use serde_json::map::Entry;

use crate::routing::{edge::DirectedEdge, types::VertexId};

use super::label_entry::LabelEntry;

#[derive(Clone, Serialize, Deserialize)]
pub struct Label {
    pub entries: Vec<LabelEntry>,
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

        Label { entries: labels }
    }

    pub fn sort_and_clean(&mut self) {
        let mut map: HashMap<VertexId, (u32, VertexId)> = HashMap::new();

        // Assuming the rest of your struct and context is defined elsewhere
        self.entries.iter().for_each(|entry| {
            // Use the entry API to access the map more efficiently
            map.entry(entry.id)
                .and_modify(|e| {
                    // Only update if the new cost is lower
                    if entry.cost < e.0 {
                        e.0 = entry.cost;
                        e.1 = entry.predecessor;
                    }
                })
                // Insert if the key does not exist
                .or_insert((entry.cost, entry.predecessor));
        });

        self.entries = map
            .iter()
            .map(|(id, cost_predecessor)| LabelEntry {
                id: *id,
                cost: cost_predecessor.0,
                predecessor: cost_predecessor.1,
            })
            .collect();

        self.entries.par_sort_unstable_by_key(|entry| entry.id);
    }

    pub fn prune_forward(&mut self, backward_labels: &Vec<Label>) {
        self.entries = self
            .entries
            .par_iter()
            .filter(|entry| {
                let backward_label = backward_labels.get(entry.id as usize).unwrap();
                let true_cost = self.get_cost(backward_label).unwrap();
                entry.cost == true_cost
            })
            .cloned()
            .collect();
    }

    pub fn prune_backward(&mut self, forward_label: &Vec<Label>) {
        self.entries = self
            .entries
            .par_iter()
            .filter(|entry| {
                let forward_label = forward_label.get(entry.id as usize).unwrap();
                let true_cost = self.get_cost(forward_label).unwrap();
                entry.cost == true_cost
            })
            .cloned()
            .collect();
    }

    pub fn set_predecessor(&mut self) {
        // create map id -> idx
        let mut id_idx = HashMap::with_capacity(self.entries.len());
        for idx in 0..self.entries.len() {
            id_idx.insert(self.entries[idx].id, idx as u32);
        }

        // use map to set id (of predecessor) to idx (of predecessor)
        for entry in self.entries.iter_mut() {
            entry.predecessor = *id_idx.get(&entry.predecessor).unwrap();
        }
    }

    pub fn get_subroute(&self, i_self: u32) -> Vec<u32> {
        let mut route = Vec::new();
        let mut idx = i_self;

        // only guaranted to terminate if set_predecessor was called before
        route.push(self.entries[idx as usize].id);
        while self.entries[idx as usize].predecessor != idx {
            idx = self.entries[idx as usize].predecessor;
            route.push(self.entries[idx as usize].id);
        }

        route
    }

    // cost, route_with_shortcuts
    pub fn get_route(&self, other: &Label) -> Option<(u32, Vec<u32>)> {
        let (cost, i_self, i_other) = self.get_idx(other)?;
        let mut f_route = self.get_subroute(i_self);
        let b_route = other.get_subroute(i_other);

        if f_route.first() == b_route.first() {
            f_route.remove(0);
        }

        f_route.reverse();
        f_route.extend(b_route);

        Some((cost, f_route))
    }

    pub fn get_cost(&self, other: &Label) -> Option<u32> {
        Some(self.get_idx(other)?.0)
    }

    /// cost, i_self, i_other
    pub fn get_idx(&self, other: &Label) -> Option<(u32, u32, u32)> {
        let mut i_self = 0;
        let mut i_other = 0;

        let mut cost = u32::MAX;
        let mut min_i_self = 0;
        let mut min_i_other = 0;

        while i_self < self.entries.len() && i_other < other.entries.len() {
            let self_entry = &self.entries[i_self];
            let other_entry = &other.entries[i_other];

            match self_entry.id.cmp(&other_entry.id) {
                std::cmp::Ordering::Less => i_self += 1,
                std::cmp::Ordering::Equal => {
                    let alternative_cost = self_entry.cost + other_entry.cost;
                    if alternative_cost < cost {
                        cost = alternative_cost;
                        min_i_self = i_self;
                        min_i_other = i_other;
                    }

                    i_self += 1;
                    i_other += 1;
                }
                std::cmp::Ordering::Greater => i_other += 1,
            }
        }

        if cost != u32::MAX {
            return Some((cost, min_i_self as u32, min_i_other as u32));
        }

        None
    }
}
