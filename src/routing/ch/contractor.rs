use indicatif::ProgressBar;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde_derive::{Deserialize, Serialize};

use crate::routing::graph::{Edge, Graph};

use super::{ch_queue::queue::CHQueue, contraction_helper::ContractionHelper};

#[derive(Clone, Serialize, Deserialize)]
pub struct ContractedGraph {
    pub graph: Graph,
    pub shortcuts_map: Vec<((u32, u32), u32)>,
}

pub struct Contractor {
    graph: Graph,
    queue: CHQueue,
    levels: Vec<u32>,
}

impl Contractor {
    pub fn new(graph: &Graph) -> Self {
        let levels = vec![0; graph.forward_edges.len()];
        let graph = graph.clone();
        let queue = CHQueue::new(&graph);

        Contractor {
            graph,
            queue,
            levels,
        }
    }

    pub fn get_contracted_graph(graph: &Graph) -> ContractedGraph {
        let mut contractor = Contractor::new(graph);
        contractor.get_graph()
    }

    pub fn get_graph(&mut self) -> ContractedGraph {
        let outgoing_edges = self.graph.forward_edges.clone();
        let incoming_edges = self.graph.backward_edges.clone();

        let shortcuts = self.contract_node_sets();

        self.graph.forward_edges = outgoing_edges;
        self.graph.backward_edges = incoming_edges;
        self.add_shortcuts(&shortcuts);
        self.removing_edges_violating_level_property();

        let map = shortcuts
            .into_iter()
            .map(|(shortcut, edges)| ((shortcut.source, shortcut.target), edges[0].target))
            .collect();

        ContractedGraph {
            graph: self.graph.clone(),
            shortcuts_map: map,
        }
    }

    /// Generates contraction hierarchy where one node at a time is contracted.
    pub fn contract_single_nodes(&mut self) -> Vec<(Edge, Vec<Edge>)> {
        let mut shortcuts = Vec::new();

        let bar = ProgressBar::new(self.graph.forward_edges.len() as u64);

        let mut level = 0;
        while let Some(v) = self.queue.pop(&self.graph) {
            let shortcut_generator = ContractionHelper::new(&self.graph);
            let mut this_shortcuts = shortcut_generator.generate_shortcuts(v, 10);

            self.add_shortcuts(&this_shortcuts);
            shortcuts.append(&mut this_shortcuts);

            self.graph.disconnect(v);
            self.levels[v as usize] = level;

            level += 1;
            bar.inc(1);
        }
        bar.finish();

        shortcuts
    }

    /// Generates contraction hierarchy where nodes from independent node sets are contracted
    /// simultainously.
    pub fn contract_node_sets(&mut self) -> Vec<(Edge, Vec<Edge>)> {
        let mut shortcuts = Vec::new();

        let bar = ProgressBar::new(self.graph.forward_edges.len() as u64);

        let mut level = 0;
        while let Some(node_set) = self.queue.pop_vec(&self.graph) {
            let shortcut_generator = ContractionHelper::new(&self.graph);
            let mut this_shortcuts = node_set
                .par_iter()
                .map(|&v| shortcut_generator.generate_shortcuts(v, 10))
                .flatten()
                .collect::<Vec<_>>();

            self.add_shortcuts(&this_shortcuts);
            shortcuts.append(&mut this_shortcuts);

            for &v in node_set.iter() {
                self.graph.disconnect(v);
                self.levels[v as usize] = level;
            }

            bar.inc(node_set.len() as u64);
            level += 1;
        }
        bar.finish();

        shortcuts
    }

    fn add_shortcuts(&mut self, shortcuts: &Vec<(Edge, Vec<Edge>)>) {
        for (shortcut, _) in shortcuts {
            self.graph.forward_edges[shortcut.source as usize].push(shortcut.clone());
            self.graph.backward_edges[shortcut.target as usize].push(shortcut.clone());
        }
    }

    pub fn removing_edges_violating_level_property(&mut self) {
        self.graph.forward_edges.iter_mut().for_each(|edges| {
            edges.retain(|edge| {
                self.levels[edge.source as usize] <= self.levels[edge.target as usize]
            });
        });

        self.graph.backward_edges.iter_mut().for_each(|edges| {
            edges.retain(|edge| {
                self.levels[edge.source as usize] >= self.levels[edge.target as usize]
            });
        });
    }
}
