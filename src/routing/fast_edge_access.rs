use super::{
    edge::{DirectedHeadlessWeightedEdge, DirectedTaillessWeightedEdge, DirectedWeightedEdge},
    types::VertexId,
};

#[derive(Clone)]
pub struct FastOutEdgeAccess {
    pub edges: Vec<DirectedTaillessWeightedEdge>,
    pub tail_start_at: Vec<u32>,
}

impl FastOutEdgeAccess {
    pub fn new(edges: &[DirectedWeightedEdge]) -> FastOutEdgeAccess {
        let mut edges = edges.to_vec();
        let mut edges_start_at: Vec<u32> = vec![0; edges.len() + 1];

        // temporarrly adding a node in order to generate the list
        edges.push(DirectedWeightedEdge {
            tail: edges.len() as u32,
            head: 0,
            cost: 0,
        });
        edges.sort_unstable_by_key(|edge| edge.tail);

        let mut current_tail = 0;
        for (edge_idx, edge) in edges.iter().enumerate() {
            if edge.tail != current_tail {
                for index in (current_tail + 1)..=edge.tail {
                    edges_start_at[index as usize] = edge_idx as u32;
                }
                current_tail = edge.tail;
            }
        }
        edges.pop();
        let edges: Vec<_> = edges.iter().map(|edge| edge.get_out_fast_edge()).collect();

        FastOutEdgeAccess {
            edges,
            tail_start_at: edges_start_at,
        }
    }

    pub fn edges(&self, source: VertexId) -> &[DirectedTaillessWeightedEdge] {
        let start = self.tail_start_at[source as usize] as usize;
        let end = self.tail_start_at[source as usize + 1] as usize;

        &self.edges[start..end]
    }
}

#[derive(Clone)]
pub struct FastInEdgeAccess {
    pub edges: Vec<DirectedHeadlessWeightedEdge>,
    pub head_start_at: Vec<u32>,
}

impl FastInEdgeAccess {
    pub fn new(edges: &[DirectedWeightedEdge]) -> FastInEdgeAccess {
        let mut edges: Vec<_> = edges.to_vec();
        let mut edges_start_at: Vec<u32> = vec![0; edges.len() + 1];

        // temporarrly adding a node in order to generate the list
        edges.push(DirectedWeightedEdge {
            tail: 0,
            head: edges.len() as u32,
            cost: 0,
        });
        edges.sort_unstable_by_key(|edge| edge.head);

        let mut current_head = 0;
        for (edge_idx, edge) in edges.iter().enumerate() {
            if edge.head != current_head {
                for index in (current_head + 1)..=edge.head {
                    edges_start_at[index as usize] = edge_idx as u32;
                }
                current_head = edge.head;
            }
        }
        edges.pop();
        let edges: Vec<_> = edges.iter().map(|edge| edge.get_in_fast_edge()).collect();

        FastInEdgeAccess {
            edges,
            head_start_at: edges_start_at,
        }
    }

    pub fn edges(&self, source: u32) -> &[DirectedHeadlessWeightedEdge] {
        let start = self.head_start_at[source as usize] as usize;
        let end = self.head_start_at[source as usize + 1] as usize;

        &self.edges[start..end]
    }
}
