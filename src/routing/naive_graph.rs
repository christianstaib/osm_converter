use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead},
};

use crate::sphere::geometry::point::Point;

use super::graph::Edge;

#[derive(Clone)]
pub struct NaiveGraph {
    pub nodes: Vec<Point>,
    pub edges: Vec<Edge>,
}

impl NaiveGraph {
    pub fn from_file(filename: &str) -> NaiveGraph {
        let file = File::open(filename).unwrap();
        let reader = io::BufReader::new(file);

        let mut lines = reader.lines();
        let number_of_nodes: usize = lines.by_ref().next().unwrap().unwrap().parse().unwrap();
        let number_of_edges: usize = lines.by_ref().next().unwrap().unwrap().parse().unwrap();

        let nodes: Vec<_> = lines
            .by_ref()
            .take(number_of_nodes)
            .map(|node_line| {
                // nodeID nodeID2 latitude longitude elevation
                let node_line = node_line.unwrap();
                let mut values = node_line.split_whitespace();
                values.next();
                values.next();
                let latitude: f64 = values.next().unwrap().parse().unwrap();
                let longitude: f64 = values.next().unwrap().parse().unwrap();
                values.next();
                Point::from_coordinate(latitude, longitude)
            })
            .collect();

        let edges: Vec<_> = lines
            .by_ref()
            .take(number_of_edges)
            .map(|edge_line| {
                // srcIDX trgIDX cost type maxspeed
                let line = edge_line.unwrap();
                let mut values = line.split_whitespace();
                let source: u32 = values.next().unwrap().parse().unwrap();
                let target: u32 = values.next().unwrap().parse().unwrap();
                let cost: u32 = values.next().unwrap().parse().unwrap();
                values.next();
                values.next();
                Edge::new(source, target, cost)
            })
            .collect();

        NaiveGraph { nodes, edges }
    }

    // pub fn _make_bidirectional(&mut self) {
    //     let mut edge_map = HashMap::new();
    //     self.edges.iter().for_each(|edge| {
    //         let key = (edge.source, edge.target);
    //         let key = (std::cmp::min(key.0, key.1), std::cmp::max(key.0, key.1));
    //         if &edge.cost < edge_map.get(&key).unwrap_or(&u32::MAX) {
    //             edge_map.insert(key, edge.cost);
    //         }
    //     });
    //     self.edges = edge_map
    //         .iter()
    //         .map(|(&(source, target), &cost)| Edge::new(source, target, cost))
    //         .flat_map(|edge| vec![edge.clone(), edge.get_inverted()])
    //         .collect();
    // }
}
