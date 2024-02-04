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
    pub fn new(edges: &[Vec<DirectedWeightedEdge>]) -> FastOutEdgeAccess {
        let mut edges_start_at = vec![0];

        for edges in edges.iter() {
            edges_start_at.push(edges_start_at.last().unwrap() + edges.len() as u32);
        }

        let edges = edges
            .iter()
            .flatten()
            .map(|edge| edge.get_out_fast_edge())
            .collect();

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
    pub fn new(edges: &[Vec<DirectedWeightedEdge>]) -> FastInEdgeAccess {
        let mut edges_start_at = vec![0];

        for edges in edges.iter() {
            edges_start_at.push(edges_start_at.last().unwrap() + edges.len() as u32);
        }

        let edges = edges
            .iter()
            .flatten()
            .map(|edge| edge.get_in_fast_edge())
            .collect();

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
