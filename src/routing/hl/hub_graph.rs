use indicatif::{ParallelProgressIterator, ProgressBar, ProgressIterator, ProgressStyle};
use rayon::{
    iter::{IntoParallelIterator, ParallelIterator},
    prelude::IntoParallelRefMutIterator,
};
use serde_derive::{Deserialize, Serialize};

use crate::routing::{route::RouteRequest, simple_algorithms::ch_bi_dijkstra::ChDijkstra};

use super::label::Label;

#[derive(Serialize, Deserialize)]
pub struct HubGraph {
    pub forward_labels: Vec<Label>,
    pub backward_labels: Vec<Label>,
}

impl HubGraph {
    pub fn new(dijkstra: &ChDijkstra, depth_limit: u32) -> HubGraph {
        let style =
            ProgressStyle::with_template("{wide_bar} {human_pos}/{human_len} {eta_precise}")
                .unwrap();
        let pb = ProgressBar::new((dijkstra.graph.num_nodes * 2) as u64);
        pb.set_style(style);
        let forward_labels = (0..dijkstra.graph.num_nodes)
            .into_par_iter()
            .progress_with(pb.clone())
            .map(|id| Label::new(&dijkstra.get_forward_label(id, depth_limit)))
            .collect();
        pb.set_position(dijkstra.graph.num_nodes as u64);
        let backward_labels = (0..dijkstra.graph.num_nodes)
            .into_par_iter()
            .progress_with(pb)
            .map(|id| Label::new(&dijkstra.get_backward_label(id, depth_limit)))
            .collect();
        HubGraph {
            forward_labels,
            backward_labels,
        }
    }

    pub fn get_avg_label_size(&self) -> f32 {
        let summed_label_size: u64 = self
            .forward_labels
            .iter()
            .map(|label| label.label.len() as u64)
            .sum::<u64>()
            + self
                .backward_labels
                .iter()
                .map(|label| label.label.len() as u64)
                .sum::<u64>();
        summed_label_size as f32 / (2 * self.forward_labels.len()) as f32
    }

    pub fn prune(&mut self) {
        let style =
            ProgressStyle::with_template("{wide_bar} {human_pos}/{human_len} {eta_precise}")
                .unwrap();
        let pb = ProgressBar::new((self.forward_labels.len() * 2) as u64);
        pb.set_style(style);
        self.forward_labels
            .par_iter_mut()
            .progress_with(pb.clone())
            .for_each(|forward_label| forward_label.prune_forward(&self.backward_labels));
        pb.set_position(self.forward_labels.len() as u64);
        self.backward_labels
            .par_iter_mut()
            .progress_with(pb)
            .for_each(|backward_label| backward_label.prune_forward(&self.forward_labels));
    }

    pub fn set_predecessor(&mut self) {
        let style =
            ProgressStyle::with_template("{wide_bar} {human_pos}/{human_len} {eta_precise}")
                .unwrap();
        let pb = ProgressBar::new((self.forward_labels.len() * 2) as u64);
        pb.set_style(style);
        self.forward_labels
            .par_iter_mut()
            .progress_with(pb.clone())
            .for_each(|label| label.set_predecessor());
        pb.set_position(self.forward_labels.len() as u64);
        self.backward_labels
            .par_iter_mut()
            .progress_with(pb.clone())
            .for_each(|label| label.set_predecessor());
    }

    pub fn get_cost(&self, request: &RouteRequest) -> Option<u32> {
        let forward_label = self.forward_labels.get(request.source as usize)?;
        let backward_label = self.backward_labels.get(request.target as usize)?;
        forward_label.get_cost(backward_label)
    }
}
