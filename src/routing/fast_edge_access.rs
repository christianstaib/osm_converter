use super::{
    edge::{DirectedHeadlessWeightedEdge, DirectedTaillessWeightedEdge, DirectedWeightedEdge},
    types::VertexId,
};

#[derive(Clone)]
pub struct FastOutEdgeAccess {
    pub edges: Vec<DirectedTaillessWeightedEdge>,
    pub edges_start_at: Vec<u32>,
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

        let mut current = 0;
        for (i, edge) in edges.iter().enumerate() {
            if edge.tail != current {
                for index in (current + 1)..=edge.tail {
                    edges_start_at[index as usize] = i as u32;
                }
                current = edge.tail;
            }
        }
        edges.pop();
        let edges: Vec<_> = edges.iter().map(|edge| edge.get_out_fast_edge()).collect();

        FastOutEdgeAccess {
            edges,
            edges_start_at,
        }
    }

    pub fn edges(&self, source: VertexId) -> &[DirectedTaillessWeightedEdge] {
        let start = self.edges_start_at[source as usize] as usize;
        let end = self.edges_start_at[source as usize + 1] as usize;

        &self.edges[start..end]
    }
}

#[derive(Clone)]
pub struct FastInEdgeAccess {
    pub edges: Vec<DirectedHeadlessWeightedEdge>,
    pub edges_start_at: Vec<u32>,
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

        let mut current = 0;
        for (i, edge) in edges.iter().enumerate() {
            if edge.head != current {
                for index in (current + 1)..=edge.head {
                    edges_start_at[index as usize] = i as u32;
                }
                current = edge.head;
            }
        }
        edges.pop();
        let edges: Vec<_> = edges.iter().map(|edge| edge.get_in_fast_edge()).collect();

        FastInEdgeAccess {
            edges,
            edges_start_at,
        }
    }

    pub fn edges(&self, source: u32) -> &[DirectedHeadlessWeightedEdge] {
        let start = self.edges_start_at[source as usize] as usize;
        let end = self.edges_start_at[source as usize + 1] as usize;

        &self.edges[start..end]
    }
}
