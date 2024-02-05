use crate::routing::{
    dijkstra_data::DijkstraData,
    fast_graph::FastGraph,
    route::{Route, RouteRequest, RouteResponse, Routing},
    types::VertexId,
};

#[derive(Clone)]
pub struct BiDijkstra<'a> {
    pub graph: &'a FastGraph,
}

impl<'a> Routing for BiDijkstra<'a> {
    fn get_route(&self, route_request: &RouteRequest) -> RouteResponse {
        self.get_data(&route_request)
    }
}

impl<'a> BiDijkstra<'a> {
    pub fn new(graph: &'a FastGraph) -> BiDijkstra {
        BiDijkstra { graph }
    }

    pub fn get_data(&self, request: &RouteRequest) -> RouteResponse {
        let mut forward_data = DijkstraData::new(self.graph.num_nodes as usize, request.source);
        let mut backward_data = DijkstraData::new(self.graph.num_nodes as usize, request.target);

        let route = self.get_route_data(&mut forward_data, &mut backward_data);

        RouteResponse {
            route,
            data: vec![forward_data, backward_data],
        }
    }

    pub fn get_route_data(
        &self,
        forward_data: &mut DijkstraData,
        backward_data: &mut DijkstraData,
    ) -> Option<Route> {
        let mut minimal_cost = u32::MAX;
        let mut minimal_cost_vertex = u32::MAX;

        loop {
            let forward_state = forward_data.pop();
            if let Some(forward_state) = forward_state {
                if backward_data.verticies[forward_state.value as usize].is_expanded {
                    let contact_cost = forward_data.verticies[forward_state.value as usize].cost
                        + backward_data.verticies[forward_state.value as usize].cost;
                    if contact_cost < minimal_cost {
                        minimal_cost = contact_cost;
                        minimal_cost_vertex = forward_state.value;
                    }
                }
                self.graph
                    .out_edges(forward_state.value)
                    .iter()
                    .for_each(|edge| {
                        forward_data.update(forward_state.value, edge.head, edge.cost)
                    });
            }

            let backward_state = backward_data.pop();
            if let Some(backward_state) = backward_state {
                if forward_data.verticies[backward_state.value as usize].is_expanded {
                    let contact_cost = forward_data.verticies[backward_state.value as usize].cost
                        + backward_data.verticies[backward_state.value as usize].cost;
                    if contact_cost < minimal_cost {
                        minimal_cost = contact_cost;
                        minimal_cost_vertex = backward_state.value;
                    }
                }
                self.graph
                    .in_edges(backward_state.value)
                    .iter()
                    .for_each(|edge| {
                        backward_data.update(backward_state.value, edge.tail, edge.cost);
                    });
            }

            if forward_state.is_none() && backward_state.is_none() {
                break;
            }
        }

        construct_route(minimal_cost_vertex, forward_data, backward_data)
    }
}

fn construct_route(
    contact_node: VertexId,
    forward_data: &DijkstraData,
    backward_data: &DijkstraData,
) -> Option<Route> {
    let mut forward_route = forward_data.get_route(contact_node)?;
    let mut backward_route = backward_data.get_route(contact_node)?;
    backward_route.nodes.pop();
    backward_route.nodes.reverse();
    forward_route.nodes.extend(backward_route.nodes);
    forward_route.cost += backward_route.cost;

    Some(forward_route)
}
