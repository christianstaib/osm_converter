use ahash::HashMap;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::{
    iter::{IntoParallelIterator, ParallelIterator},
    prelude::IntoParallelRefMutIterator,
};
use serde_derive::{Deserialize, Serialize};

use crate::routing::{
    edge::DirectedEdge,
    path::{Path, PathRequest},
    simple_algorithms::ch_bi_dijkstra::ChDijkstra,
    types::Weight,
};

use super::label::Label;

#[derive(Serialize, Deserialize)]
pub struct HubGraph {
    pub forward_labels: Vec<Label>,
    pub reverse_labels: Vec<Label>,
    pub shortcuts: HashMap<DirectedEdge, u32>,
}

impl HubGraph {
    pub fn new(dijkstra: &ChDijkstra, depth_limit: u32) -> HubGraph {
        let forward_labels = (0..dijkstra.graph.num_nodes())
            .into_par_iter()
            .progress()
            .map(|id| Label::new(&dijkstra.get_forward_label(id, depth_limit)))
            .collect();
        let reverse_labels = (0..dijkstra.graph.num_nodes())
            .into_par_iter()
            .progress()
            .map(|id| Label::new(&dijkstra.get_backward_label(id, depth_limit)))
            .collect();

        HubGraph {
            forward_labels,
            reverse_labels,
            shortcuts: dijkstra.shortcuts.clone(),
        }
    }

    pub fn get_avg_label_size(&self) -> f32 {
        let summed_label_size: u64 = self
            .forward_labels
            .iter()
            .map(|label| label.entries.len() as u64)
            .sum::<u64>()
            + self
                .reverse_labels
                .iter()
                .map(|label| label.entries.len() as u64)
                .sum::<u64>();
        summed_label_size as f32 / (2 * self.forward_labels.len()) as f32
    }

    pub fn set_predecessor(&mut self) {
        self.forward_labels
            .par_iter_mut()
            .chain(self.reverse_labels.par_iter_mut())
            .progress()
            .for_each(|label| label.set_predecessor());
    }

    pub fn get_cost(&self, request: &PathRequest) -> Option<u32> {
        let forward_label = self.forward_labels.get(request.source as usize)?;
        let backward_label = self.reverse_labels.get(request.target as usize)?;
        Self::get_weight(forward_label, backward_label)
    }

    pub fn get_route(&self, request: &PathRequest) -> Option<Path> {
        let forward_label = self.forward_labels.get(request.source as usize)?;
        let backward_label = self.reverse_labels.get(request.target as usize)?;
        let mut path_with_shortcuts = Self::get_path_with_shortcuts(forward_label, backward_label)?;

        let mut path = Path {
            verticies: Vec::new(),
            cost: path_with_shortcuts.cost,
        };

        while path_with_shortcuts.verticies.len() >= 2 {
            let last_num = path_with_shortcuts.verticies.pop().unwrap();
            let second_last_num = *path_with_shortcuts.verticies.last().unwrap();
            let last = DirectedEdge {
                tail: second_last_num,
                head: last_num,
            };
            if let Some(&middle_node) = self.shortcuts.get(&last) {
                path_with_shortcuts
                    .verticies
                    .extend([middle_node, last.head]);
            } else {
                path.verticies.push(last.head);
            }
        }

        path.verticies.push(path_with_shortcuts.verticies[0]);
        path.verticies.reverse();

        Some(path)
    }

    // cost, route_with_shortcuts
    pub fn get_path_with_shortcuts(forward: &Label, reverse: &Label) -> Option<Path> {
        let (cost, forward_idx, reverse_idx) = Self::get_overlap(forward, reverse)?;
        let mut f_route = forward.get_path(forward_idx);
        let b_route = reverse.get_path(reverse_idx);

        if f_route.first() == b_route.first() {
            f_route.remove(0);
        }

        f_route.reverse();
        f_route.extend(b_route);

        Some(Path {
            verticies: f_route,
            cost,
        })
    }

    pub fn get_weight(forward: &Label, reverse: &Label) -> Option<u32> {
        let (weight, _, _) = Self::get_overlap(forward, reverse)?;
        Some(weight)
    }

    /// cost, i_self, i_other
    pub fn get_overlap(forward: &Label, reverse: &Label) -> Option<(Weight, u32, u32)> {
        let mut i_forward = 0;
        let mut i_reverse = 0;

        let mut overlap_weight = None;
        let mut overlap_i_forward = 0;
        let mut overlap_i_reverse = 0;

        while i_forward < forward.entries.len() && i_reverse < reverse.entries.len() {
            let forward_entry = &forward.entries[i_forward];
            let reverse_entry = &reverse.entries[i_reverse];

            match forward_entry.id.cmp(&reverse_entry.id) {
                std::cmp::Ordering::Less => i_forward += 1,
                std::cmp::Ordering::Equal => {
                    let alternative_weight =
                        forward_entry.cost.checked_add(reverse_entry.cost).unwrap();
                    if alternative_weight < overlap_weight.unwrap_or(u32::MAX) {
                        overlap_weight = Some(alternative_weight);
                        overlap_i_forward = i_forward;
                        overlap_i_reverse = i_reverse;
                    }

                    i_forward += 1;
                    i_reverse += 1;
                }
                std::cmp::Ordering::Greater => i_reverse += 1,
            }
        }

        if let Some(min_weight) = overlap_weight {
            return Some((
                min_weight,
                overlap_i_forward as u32,
                overlap_i_reverse as u32,
            ));
        }

        None
    }
}
