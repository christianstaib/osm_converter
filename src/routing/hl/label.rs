use std::usize;

use ahash::{HashMap, HashMapExt};
use rayon::{
    iter::{IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSliceMut,
};
use serde_derive::{Deserialize, Serialize};

use crate::routing::types::VertexId;

use super::{hub_graph::HubGraph, label_entry::LabelEntry};

#[derive(Clone, Serialize, Deserialize)]
pub struct Label {
    pub entries: Vec<LabelEntry>,
}

impl Label {
    pub fn sort_and_clean(&mut self) {
        let mut map: HashMap<VertexId, (u32, Option<VertexId>)> = HashMap::new();

        // Assuming the rest of your struct and context is defined elsewhere
        self.entries.iter().for_each(|entry| {
            // Use the entry API to access the map more efficiently
            map.entry(entry.vertex)
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
                vertex: *id,
                cost: cost_predecessor.0,
                predecessor: cost_predecessor.1,
            })
            .collect();

        self.entries.par_sort_unstable_by_key(|entry| entry.vertex);
    }

    pub fn prune_forward(&mut self, backward_labels: &Vec<Label>) {
        self.entries = self
            .entries
            .par_iter()
            .filter(|entry| {
                let backward_label = backward_labels.get(entry.vertex as usize).unwrap();
                let true_cost = HubGraph::get_weight_labels(self, backward_label).unwrap();
                entry.cost == true_cost
            })
            .cloned()
            .collect();
    }

    pub fn prune_backward(&mut self, forward_labels: &Vec<Label>) {
        self.entries = self
            .entries
            .par_iter()
            .filter(|entry| {
                let forward_label = forward_labels.get(entry.vertex as usize).unwrap();
                let true_cost = HubGraph::get_weight_labels(forward_label, self).unwrap();
                entry.cost == true_cost
            })
            .cloned()
            .collect();
    }

    pub fn set_predecessor(&mut self) {
        // maps vertex -> index
        let mut id_idx = HashMap::with_capacity(self.entries.len());
        for idx in 0..self.entries.len() {
            id_idx.insert(self.entries[idx].vertex, idx as u32);
        }

        for entry in self.entries.iter_mut() {
            if let Some(predecessor) = entry.predecessor {
                entry.predecessor = Some(*id_idx.get(&predecessor).unwrap());
            }
        }
    }

    pub fn get_path(&self, edge_id: u32) -> Vec<u32> {
        let mut route = Vec::new();
        let mut idx = edge_id;

        // only guaranted to terminate if set_predecessor was called before
        route.push(self.entries[idx as usize].vertex);
        while let Some(this_idx) = self.entries[idx as usize].predecessor {
            idx = this_idx;
            route.push(self.entries[idx as usize].vertex);
        }

        route
    }
}
