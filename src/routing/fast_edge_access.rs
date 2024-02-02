use super::{fast_graph::FastOutEdge, graph::DirectedEdge};

#[derive(Clone)]
pub struct FastEdgeAccess {
    pub edges: Vec<FastOutEdge>,
    pub edges_start_at: Vec<u32>,
}

impl FastEdgeAccess {
    pub fn new(edges: &Vec<DirectedEdge>) -> FastEdgeAccess {
        let mut edges = edges.clone();

        let mut edges_start_at: Vec<u32> = vec![0; edges.len() + 1];

        // temporarrly adding a node in order to generate the list
        edges.push(DirectedEdge {
            head: edges.len() as u32,
            tail: 0,
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
        let edges: Vec<_> = edges.iter().map(|edge| edge.get_fast_edge()).collect();
        let edges_start_at = edges_start_at.clone();

        FastEdgeAccess {
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
