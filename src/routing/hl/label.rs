use core::panic;
use std::usize;

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde_derive::{Deserialize, Serialize};

use crate::routing::path::Path;

use super::{hub_graph::HubGraph, label_entry::LabelEntry};

#[derive(Clone, Serialize, Deserialize)]
pub struct Label {
    pub entries: Vec<LabelEntry>,
}

impl Label {
    pub fn clean(&mut self) {
        let old_entries = std::mem::take(&mut self.entries);

        old_entries.into_iter().for_each(|old_entry| {
            let search_result = self
                .entries
                .binary_search_by_key(&old_entry.vertex, |self_entry| self_entry.vertex);

            match search_result {
                Ok(idx) => {
                    if old_entry.weight < self.entries[idx as usize].weight {
                        self.entries[idx as usize].weight = old_entry.weight;
                        self.entries[idx as usize].predecessor = old_entry.predecessor;
                    }
                }
                Err(idx) => self.entries.insert(idx, old_entry),
            }
        });
    }

    pub fn prune_forward_label(&mut self, reverse_labels: &[Label]) {
        self.entries = self
            .entries
            .par_iter()
            .filter(|entry| {
                let reverse_label = &reverse_labels[entry.vertex as usize];
                let true_cost = HubGraph::get_weight_labels(self, reverse_label).unwrap();
                entry.weight == true_cost
            })
            .cloned()
            .collect();
    }

    pub fn prune_reverse_label(&mut self, forward_labels: &[Label]) {
        self.entries = self
            .entries
            .par_iter()
            .filter(|entry| {
                let forward_label = &forward_labels[entry.vertex as usize];
                let true_cost = HubGraph::get_weight_labels(forward_label, self).unwrap();
                entry.weight == true_cost
            })
            .cloned()
            .collect();
    }

    pub fn set_predecessor(&mut self) {
        // maps vertex -> index
        let mut vertex_to_index = HashMap::new();
        for idx in 0..self.entries.len() {
            vertex_to_index.insert(self.entries[idx].vertex, idx as u32);
        }

        // replace predecessor VertexId with index of predecessor
        for entry in self.entries.iter_mut() {
            if let Some(predecessor) = entry.predecessor {
                entry.predecessor = Some(*vertex_to_index.get(&predecessor).unwrap());
            }
        }
    }

    pub fn get_path(&self, edge_id: u32) -> Path {
        let mut path = Path {
            vertices: Vec::new(),
            weight: self.entries[edge_id as usize].weight,
        };
        let mut idx = edge_id;
        let mut visited = HashSet::new();

        while let Some(entry) = self.entries.get(idx as usize) {
            // cycle detection
            if !visited.insert(idx) {
                panic!("wrong formated label");
            }

            path.vertices.push(entry.vertex);

            if let Some(this_idx) = entry.predecessor {
                idx = this_idx;
            } else {
                // exit the loop if there's no predecessor
                break;
            }
        }

        path
    }
}
