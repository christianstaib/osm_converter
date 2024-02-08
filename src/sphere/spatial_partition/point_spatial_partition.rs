use indicatif::ProgressIterator;

use crate::sphere::geometry::{
    collision_detection::{Collides, Contains},
    point::Point,
};

use super::tiling::{ConvecQuadrilateral, Tiling};

#[derive(Clone)]
pub struct PointSpatialPartition {
    boundary: ConvecQuadrilateral,
    node_type: PointNodeType,
    max_size: usize,
}

#[derive(Clone)]
enum PointNodeType {
    Internal(Vec<PointSpatialPartition>), // four children
    Leaf(Vec<Point>),                     // a bucket of points
}

impl PointSpatialPartition {
    pub fn new_root(max_size: usize) -> PointSpatialPartition {
        let boundary = ConvecQuadrilateral::new(&vec![
            Point::from_coordinate(0.0, 0.0),
            Point::from_coordinate(1.0, 1.0),
            Point::from_coordinate(1.0, -1.0),
            Point::from_coordinate(1.0, -1.0),
            Point::from_coordinate(0.0, 0.0),
        ]);
        PointSpatialPartition {
            boundary,
            node_type: PointNodeType::Internal(
                Tiling::base_tiling()
                    .iter()
                    .cloned()
                    .map(|p| PointSpatialPartition::new_leaf(p, max_size))
                    .collect(),
            ),
            max_size,
        }
    }

    fn new_leaf(boundary: ConvecQuadrilateral, max_size: usize) -> PointSpatialPartition {
        PointSpatialPartition {
            boundary,
            node_type: PointNodeType::Leaf(Vec::with_capacity(max_size + 1)),
            max_size,
        }
    }

    fn split(&mut self) {
        let mut points: Vec<Point> = Vec::new();
        if let PointNodeType::Leaf(old_points) = &self.node_type {
            points.extend(old_points);
        }
        self.node_type = PointNodeType::Internal(
            self.boundary
                .split()
                .into_iter()
                .map(|rectangle| PointSpatialPartition::new_leaf(rectangle, self.max_size))
                .collect(),
        );

        points.iter().for_each(|point| self.add_point(point));
    }

    pub fn add_points(&mut self, points: &Vec<Point>) {
        println!("len is {}", points.len());
        points
            .iter()
            .progress()
            .for_each(|point| self.add_point(point));
    }

    pub fn add_point(&mut self, point: &Point) {
        let mut internals = vec![self];
        while let Some(parent) = internals.pop() {
            // needs to be done before the match block, as the match block pushes a mutable
            // reference to internals.
            if let PointNodeType::Leaf(points) = &mut parent.node_type {
                points.push(*point);
                if points.len() >= parent.max_size {
                    parent.split();
                }
                break;
            } else if let PointNodeType::Internal(childs) = &mut parent.node_type {
                for child in childs.iter_mut() {
                    if child.boundary.contains(point) {
                        internals.push(child);
                        break;
                    }
                }
            }
        }
    }

    pub fn get_points(&self, polygon: &ConvecQuadrilateral) -> Vec<Point> {
        let mut points = Vec::new();
        let mut internals = vec![self];
        while let Some(parent) = internals.pop() {
            if let PointNodeType::Leaf(leaf_points) = &parent.node_type {
                points.extend(
                    leaf_points
                        .iter()
                        .filter(|&point| polygon.contains(point))
                        .cloned(),
                );
            } else if let PointNodeType::Internal(childs) = &parent.node_type {
                internals.extend(childs.iter().filter(|q| q.boundary.collides(polygon)));
            }
        }

        points
    }
}
