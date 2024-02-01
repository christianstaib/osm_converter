use crate::routing::{
    dijkstra_data::DijkstraData,
    fast_graph::FastGraph,
    route::{RouteRequest, RouteResponse},
};

use super::heuristics::Heuristic;

pub struct AStar<'a> {
    pub graph: &'a FastGraph,
}

impl<'a> AStar<'a> {
    pub fn new(graph: &'a FastGraph) -> AStar {
        AStar { graph }
    }

    pub fn get_data(&self, request: &RouteRequest, heuristic: Box<dyn Heuristic>) -> RouteResponse {
        let mut data = DijkstraData::new(self.graph.num_nodes as usize, request.source);

        while let Some(state) = data.pop() {
            if state.value == request.target {
                break;
            }

            self.graph.out_edges(state.value).iter().for_each(|edge| {
                let h = heuristic.lower_bound(edge.target);
                data.update(state.value, edge, h);
            })
        }

        RouteResponse {
            route: data.get_route(request.target),
            data: vec![data],
        }
    }
}
