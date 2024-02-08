use indicatif::ProgressIterator;

use crate::sphere::geometry::{
    arc::Arc,
    collision_detection::{Collides, CollisionDetection, Contains},
    point::{meters_to_radians, Point},
    polygon::Polygon,
};

use super::tiling::{ConvecQuadrilateral, Tiling};

#[derive(Clone)]
pub struct PolygonSpatialPartition {
    boundary: ConvecQuadrilateral,
    node_type: NodeType,
    max_size: usize,
    midpoint: Point,
    midpoint_flag: PointStatus,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PointStatus {
    Inside,
    Outside,
}

#[derive(Clone)]
enum NodeType {
    Internal(Vec<PolygonSpatialPartition>), // four children
    Leaf(Vec<Arc>),                         // a bucket of points
}

impl PointStatus {
    pub fn other(&self) -> PointStatus {
        match self {
            PointStatus::Inside => PointStatus::Outside,
            PointStatus::Outside => PointStatus::Inside,
        }
    }
}

impl CollisionDetection for PolygonSpatialPartition {
    fn is_on_polygon(&self, point: &Point) -> bool {
        self.is_on_polygon(point)
    }
}

impl PolygonSpatialPartition {
    pub fn new(max_size: usize) -> PolygonSpatialPartition {
        let boundary = ConvecQuadrilateral::new(&vec![
            Point::from_coordinate(0.0, 0.0),
            Point::from_coordinate(1.0, 1.0),
            Point::from_coordinate(1.0, -1.0),
            Point::from_coordinate(1.0, -1.0),
            Point::from_coordinate(0.0, 0.0),
        ]);
        let midpoint = boundary.get_midpoint();
        PolygonSpatialPartition {
            boundary,
            node_type: NodeType::Internal(
                Tiling::base_tiling()
                    .iter()
                    .cloned()
                    .map(|p| PolygonSpatialPartition::new_leaf(p, max_size))
                    .collect(),
            ),
            max_size,
            midpoint,
            midpoint_flag: PointStatus::Outside,
        }
    }

    fn new_leaf(boundary: ConvecQuadrilateral, max_size: usize) -> PolygonSpatialPartition {
        let midpoint = boundary.get_midpoint();
        PolygonSpatialPartition {
            boundary,
            node_type: NodeType::Leaf(Vec::with_capacity(max_size + 1)),
            max_size,
            midpoint,
            midpoint_flag: PointStatus::Outside,
        }
    }

    pub fn add_polygons(&mut self, polygons: &Vec<Polygon>) {
        polygons
            .iter()
            .filter(|polygon| polygon.contains(&self.midpoint))
            .for_each(|_| self.midpoint_flag = self.midpoint_flag.other());

        let number_of_arcs: u32 = polygons
            .iter()
            .map(|polygon| polygon.outline.len().saturating_sub(1) as u32)
            .sum();

        polygons
            .iter()
            .flat_map(|polygon| polygon.arcs())
            .progress_count(number_of_arcs as u64)
            .for_each(|arc| self.add_arc(&arc));

        self.update_midpoints();
    }

    fn split(&mut self) {
        let mut arcs: Vec<Arc> = Vec::new();
        if let NodeType::Leaf(old_arcs) = &mut self.node_type {
            arcs.extend(old_arcs.drain(0..));
        }
        self.node_type = NodeType::Internal(
            self.boundary
                .split()
                .into_iter()
                .map(|rectangle| PolygonSpatialPartition::new_leaf(rectangle, self.max_size))
                .collect(),
        );

        arcs.iter().for_each(|arc| self.add_arc(arc));
    }

    pub fn check_collision(&self, arc: &Arc) -> bool {
        let mut internals = vec![self];
        while let Some(parent) = internals.pop() {
            if let NodeType::Leaf(arcs) = &parent.node_type {
                if arcs.iter().any(|other| other.collides(arc)) {
                    return true;
                }
            } else if let NodeType::Internal(childs) = &parent.node_type {
                for child in childs.iter() {
                    let contrains_from = child.boundary.contains(arc.from());
                    let contrains_to = child.boundary.contains(arc.to());
                    if contrains_from && contrains_to {
                        internals.push(child);
                        break;
                    } else if child.boundary.collides(arc) {
                        // expensive check
                        internals.push(child);
                    }
                }
            }
        }
        false
    }

    fn add_arc(&mut self, arc: &Arc) {
        let mut internals = vec![self];
        while let Some(parent) = internals.pop() {
            // needs to be done before the match block, as the match block pushes a mutable
            // reference to internals.
            if let NodeType::Leaf(arcs) = &mut parent.node_type {
                arcs.push(arc.clone());
                if arcs.len() >= parent.max_size {
                    let outline =
                        Arc::new(&parent.boundary.outline[0], &parent.boundary.outline[1]);
                    if outline.central_angle() >= meters_to_radians(10.0) {
                        parent.split();
                    }
                }
            } else if let NodeType::Internal(childs) = &mut parent.node_type {
                childs.sort_by(|x, y| {
                    Arc::new(&x.midpoint, arc.from())
                        .central_angle()
                        .total_cmp(&Arc::new(&y.midpoint, arc.from()).central_angle())
                });
                for child in childs.iter_mut() {
                    let contrains_from = child.boundary.contains(arc.from());
                    let contrains_to = child.boundary.contains(arc.to());
                    if contrains_from && contrains_to {
                        internals.push(child);
                        break;
                    } else if child.boundary.collides(arc) {
                        // expensive check
                        internals.push(child);
                    }
                }
            }
        }
    }

    fn update_midpoints(&mut self) {
        let mut stack = Vec::new();
        stack.push(self);

        while let Some(current) = stack.pop() {
            let mut intersections = Vec::new();
            if let NodeType::Internal(quadtrees) = &current.node_type {
                for quadtree in quadtrees {
                    let ray = Arc::new(&current.midpoint, &quadtree.midpoint);
                    intersections.push(current.intersections(&ray).len());
                }
            }

            if let NodeType::Internal(quadtrees) = &mut current.node_type {
                for (quadtree, intersections) in quadtrees.iter_mut().zip(intersections) {
                    if intersections % 2 == 0 {
                        quadtree.midpoint_flag = current.midpoint_flag;
                    } else {
                        quadtree.midpoint_flag = current.midpoint_flag.other();
                    }
                    stack.push(quadtree);
                }
            }
        }
    }

    fn intersections(&self, ray: &Arc) -> Vec<Point> {
        let mut intersections: Vec<Point> = match &self.node_type {
            NodeType::Internal(quadtrees) => quadtrees
                .iter()
                .filter(|quadtree| {
                    quadtree.boundary.contains(ray.from())
                        || quadtree.boundary.contains(ray.to())
                        || quadtree.boundary.collides(ray)
                })
                .flat_map(|quadtree| quadtree.intersections(ray))
                .collect(),
            NodeType::Leaf(arcs) => arcs
                .iter()
                .filter_map(|arc| ray.intersection(arc))
                .collect(),
        };
        intersections.sort_by(|x, y| x.longitude().partial_cmp(&y.longitude()).unwrap());
        intersections.dedup(); // i dont know exactly why this is necesary, but it is :(
        intersections
    }

    pub fn is_on_polygon(&self, point: &Point) -> bool {
        let mut current = self;
        loop {
            match &current.node_type {
                NodeType::Internal(childs) => {
                    for child in childs {
                        let contrains = child.boundary.contains(point);
                        if contrains {
                            current = &child;
                            break;
                        }
                    }
                }
                NodeType::Leaf(arcs) => {
                    let ray = Arc::new(point, &current.midpoint);
                    let intersections = arcs.iter().filter_map(|arc| ray.intersection(arc)).count();
                    return (intersections % 2 == 0)
                        == (current.midpoint_flag == PointStatus::Inside);
                }
            }
        }
    }
}
