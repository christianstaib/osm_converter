use core::panic;
use std::usize;

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use rayon::{
    iter::{IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSliceMut,
};
use serde_derive::{Deserialize, Serialize};

use crate::routing::{
    path::Path,
    types::{VertexId, Weight},
};

use super::{hub_graph::HubGraph, label_entry::LabelEntry};

#[derive(Clone, Serialize, Deserialize)]
pub struct Label {
    pub entries: Vec<LabelEntry>,
}

impl Label {
    pub fn sort_and_clean(&mut self) {
        struct WeightPredecessor {
            weight: Weight,
            predecessor: Option<VertexId>,
        }

        // use map to remove doubled entries
        // (vertex, (weight, predecessor))
        let mut entry_map: HashMap<VertexId, WeightPredecessor> = HashMap::new();

        // Assuming the rest of your struct and context is defined elsewhere
        self.entries.iter().for_each(|self_entry| {
            // Use the entry API to access the map more efficiently

            let weight_predecessor = WeightPredecessor {
                weight: self_entry.weight,
                predecessor: self_entry.predecessor,
            };

            entry_map
                .entry(self_entry.vertex)
                .and_modify(|map_entry| {
                    // Only update if the new cost is lower
                    if self_entry.weight < map_entry.weight {
                        map_entry.weight = self_entry.weight;
                        map_entry.predecessor = self_entry.predecessor;
                    }
                })
                .or_insert(weight_predecessor);
        });

        self.entries = entry_map
            .into_iter()
            .map(|(vertex, weight_predecessor)| LabelEntry {
                vertex,
                weight: weight_predecessor.weight,
                predecessor: weight_predecessor.predecessor,
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
                entry.weight == true_cost
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
