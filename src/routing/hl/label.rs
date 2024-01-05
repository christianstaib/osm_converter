use ahash::HashMap;
use indicatif::ParallelProgressIterator;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde_derive::{Deserialize, Serialize};
use speedy::{Readable, Writable};

use crate::routing::{route::RouteRequest, simple_algorithms::ch_bi_dijkstra::ChDijkstra};

use super::label_entry::LabelEntry;

#[derive(Serialize, Deserialize, Readable, Writable)]
pub struct Label {
    pub label: Vec<LabelEntry>,
}

impl Label {
    pub fn new(map: &HashMap<u32, u32>) -> Label {
        let mut labels: Vec<_> = map
            .iter()
            .map(|(id, cost)| LabelEntry {
                id: *id,
                cost: *cost,
            })
            .collect();
        labels.sort_unstable_by_key(|entry| entry.id);
        labels.shrink_to_fit();

        Label { label: labels }
    }

    pub fn prune_forward(&mut self, source: u32, dijkstra: &ChDijkstra) {
        self.label = self
            .label
            .par_iter()
            .progress()
            .filter(|entry| {
                let request = RouteRequest {
                    source,
                    target: entry.id,
                };
                let true_cost = dijkstra.get_cost(&request).unwrap();
                entry.cost == true_cost
            })
            .cloned()
            .collect();
    }

    pub fn prune_backward(&mut self, target: u32, dijkstra: &ChDijkstra) {
        self.label = self
            .label
            .par_iter()
            .progress()
            .filter(|entry| {
                let request = RouteRequest {
                    source: entry.id,
                    target,
                };
                let true_cost = dijkstra.get_cost(&request).unwrap();
                entry.cost == true_cost
            })
            .cloned()
            .collect();
    }

    pub fn get_cost(&self, other: &Label) -> Option<u32> {
        let mut i_self = 0;
        let mut i_other = 0;

        let mut id = u32::MAX;
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
                        id = self_entry.id;
                        cost = alternative_cost;
                    }
                }
                std::cmp::Ordering::Greater => i_other += 1,
            }
        }

        if cost != u32::MAX {
            return Some(cost);
        }

        None
    }
}
