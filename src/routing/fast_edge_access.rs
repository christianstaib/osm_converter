use super::{fast_graph::FastOutEdge, graph::DirectedEdge, types::VertexId};

#[derive(Clone)]
pub struct FastOutEdgeAccess {
    pub edges: Vec<FastOutEdge>,
    pub edges_start_at: Vec<u32>,
}

impl FastOutEdgeAccess {
    pub fn new(edges: &[DirectedEdge]) -> FastOutEdgeAccess {
        let mut edges = edges.to_vec();

        let mut edges_start_at: Vec<u32> = vec![0; edges.len() + 1];

        // temporarrly adding a node in order to generate the list
        edges.push(DirectedEdge {
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
        let edges_start_at = edges_start_at.clone();

        FastOutEdgeAccess {
            edges,
            edges_start_at,
        }
    }

    pub fn edges(&self, source: VertexId) -> &[FastOutEdge] {
        let start = self.edges_start_at[source as usize] as usize;
        let end = self.edges_start_at[source as usize + 1] as usize;

        &self.edges[start..end]
    }
}

#[derive(Clone)]
pub struct FastInEdgeAccess {
    pub edges: Vec<FastOutEdge>,
    pub edges_start_at: Vec<u32>,
}

impl FastInEdgeAccess {
    pub fn new(edges: &[DirectedEdge]) -> FastInEdgeAccess {
        let mut edges: Vec<_> = edges.iter().map(|edge| edge.inverted()).collect();

        let mut edges_start_at: Vec<u32> = vec![0; edges.len() + 1];

        // temporarrly adding a node in order to generate the list
        edges.push(DirectedEdge {
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
        let edges_start_at = edges_start_at.clone();

        FastInEdgeAccess {
            edges,
            edges_start_at,
        }
    }

    pub fn edges(&self, source: u32) -> &[FastOutEdge] {
        let start = self.edges_start_at[source as usize] as usize;
        let end = self.edges_start_at[source as usize + 1] as usize;

        &self.edges[start..end]
    }
}
