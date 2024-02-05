use crate::routing::ch_graph::ChGraph;

use super::{hub_graph::HubGraph, label::Label, label_entry::LabelEntry};

pub struct HubGraphCreator {
    pub hub_graph: HubGraph,
    pub ch_graph: ChGraph,
}

impl HubGraphCreator {
    pub fn new(ch_graph: &ChGraph) -> Self {
        let labels: Vec<_> = (0..ch_graph.number_of_verticies)
            .map(|vertex| {
                let label_entry = LabelEntry {
                    id: vertex,
                    cost: 0,
                    predecessor: vertex,
                };
                Label {
                    label: vec![label_entry],
                }
            })
            .collect();

        let hub_graph = HubGraph {
            forward_labels: labels.clone(),
            backward_labels: labels,
            shortcuts: todo!(),
        };

        HubGraphCreator {
            hub_graph,
            ch_graph: ch_graph.clone(),
        }
    }

    pub fn create_hl(&mut self) {
        // for vertex in self.ch_graph
    }
}
