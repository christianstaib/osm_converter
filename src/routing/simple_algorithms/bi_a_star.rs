use crate::routing::{
    dijkstra_data::DijkstraData,
    fast_graph::FastGraph,
    route::{Route, RouteRequest, RouteResponse, Routing},
};

use super::heuristics::Heuristic;

#[derive(Clone)]
pub struct BiAStar<'a> {
    pub graph: &'a FastGraph,
}

struct ConstantHeuristic {
    forward_heuristic: Box<dyn Heuristic>,
    backward_heuristic: Box<dyn Heuristic>,
    s: u32,
    t: u32,
}

impl ConstantHeuristic {
    fn pi_f(&self, v: u32) -> u32 {
        self.forward_heuristic.lower_bound(v)
    }

    fn pi_r(&self, v: u32) -> u32 {
        self.backward_heuristic.lower_bound(v)
    }

    fn p_f(&self, v: u32) -> u32 {
        (self.pi_f(v) - self.pi_r(v)) / 2 + self.pi_r(self.t) / 2
    }

    fn p_r(&self, v: u32) -> u32 {
        (self.pi_r(v) - self.pi_f(v)) / 2 + self.pi_f(self.s) / 2
    }
}

impl<'a> BiAStar<'a> {
    pub fn new(graph: &'a FastGraph) -> BiAStar {
        BiAStar { graph }
    }

    pub fn get_data(
        &self,
        request: &RouteRequest,
        forward_heuristic: Box<dyn Heuristic>,
        backward_heuristic: Box<dyn Heuristic>,
    ) -> RouteResponse {
        let mut forward_data = DijkstraData::new(self.graph.num_nodes as usize, request.source);
        let mut backward_data = DijkstraData::new(self.graph.num_nodes as usize, request.target);

        let route = self.get_route(
            request,
            forward_heuristic,
            backward_heuristic,
            &mut forward_data,
            &mut backward_data,
        );

        RouteResponse {
            route,
            data: vec![forward_data, backward_data],
        }
    }

    pub fn get_route(
        &self,
        request: &RouteRequest,
        forward_heuristic: Box<dyn Heuristic>,
        backward_heuristic: Box<dyn Heuristic>,
        forward_data: &mut DijkstraData,
        backward_data: &mut DijkstraData,
    ) -> Option<Route> {
        let mut minimal_cost = u32::MAX;
        let mut minimal_cost_node = u32::MAX;

        let heu = ConstantHeuristic {
            forward_heuristic,
            backward_heuristic,
            s: request.source,
            t: request.target,
        };

        loop {
            let forward_state = forward_data.pop();
            if let Some(forward_state) = forward_state {
                if backward_data.nodes[forward_state.value as usize].is_expanded {
                    let contact_cost = forward_data.nodes[forward_state.value as usize].cost
                        + backward_data.nodes[forward_state.value as usize].cost;
                    if contact_cost < minimal_cost {
                        minimal_cost = contact_cost;
                        minimal_cost_node = forward_state.value;
                    }
                }
                self.graph
                    .out_edges(forward_state.value)
                    .iter()
                    .for_each(|edge| {
                        let h = heu.p_f(edge.target);
                        forward_data.update(forward_state.value, edge, h)
                    });
            }

            let backward_state = backward_data.pop();
            if let Some(backward_state) = backward_state {
                if forward_data.nodes[backward_state.value as usize].is_expanded {
                    let contact_cost = forward_data.nodes[backward_state.value as usize].cost
                        + backward_data.nodes[backward_state.value as usize].cost;
                    if contact_cost < minimal_cost {
                        minimal_cost = contact_cost;
                        minimal_cost_node = backward_state.value;
                    }
                }
                self.graph
                    .in_edges(backward_state.value)
                    .iter()
                    .for_each(|edge| {
                        let h = heu.p_r(edge.target);
                        backward_data.update(backward_state.value, edge, h);
                    });
            }

            if forward_state.is_none() && backward_state.is_none() {
                break;
            }
        }

        construct_route(minimal_cost_node, forward_data, backward_data)
    }
}

fn construct_route(
    _contact_node: u32,
    forward_data: &DijkstraData,
    backward_data: &DijkstraData,
) -> Option<Route> {
    let contact_node = forward_data
        .nodes
        .iter()
        .zip(backward_data.nodes.iter())
        .enumerate()
        .min_by_key(|(_, (forward, backward))| {
            forward.cost.checked_add(backward.cost).unwrap_or(u32::MAX)
        })
        .unwrap()
        .0 as u32;

    let mut forward_route = forward_data.get_route(contact_node)?;
    let mut backward_route = backward_data.get_route(contact_node)?;
    backward_route.nodes.pop();
    backward_route.nodes.reverse();
    forward_route.nodes.extend(backward_route.nodes);
    forward_route.cost += backward_route.cost;

    Some(forward_route)
}
