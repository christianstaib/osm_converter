use crate::sphere::spatial_partition::tiling::ConvecQuadrilateral;

use super::{
    arc::Arc,
    point::{meters_to_radians, Point},
    polygon::Polygon,
};

pub trait CollisionDetection {
    fn is_on_polygon(&self, point: &Point) -> bool;
}

//
// Collides
//

pub trait Collides<Rhs = Self> {
    /// Returns true if two objects collide. Is symmetrical.
    fn collides(&self, rhs: &Rhs) -> bool;
}

/// Define a standalone function that contains the shared logic.
fn arc_point_collision(arc: &Arc, point: &Point) -> bool {
    let summed_angle =
        Arc::new(arc.from(), point).central_angle() + Arc::new(point, arc.to()).central_angle();
    (summed_angle - arc.central_angle()).abs() < meters_to_radians(1.0)
}

fn arc_polygon_collision(arc: &Arc, polygon: &Polygon) -> bool {
    polygon.intersections(arc).is_empty()
        || polygon.contains(arc.from())
        || polygon.contains(arc.to())
}

// Arc

/// Implement `Collides<Point>` for `Arc` by using the standalone function.
impl Collides<Point> for Arc {
    fn collides(&self, rhs: &Point) -> bool {
        arc_point_collision(self, rhs)
    }
}

/// Checks if two arcs collide. Colliding does not necessarily mean that they intersect, they
/// could also be really close to another.
impl Collides<Arc> for Arc {
    fn collides(&self, rhs: &Arc) -> bool {
        self.intersects(rhs)
            || self.collides(rhs.from())
            || self.collides(rhs.to())
            || rhs.collides(self.from())
            || rhs.collides(self.to())
    }
}

impl Collides<Polygon> for Arc {
    fn collides(&self, rhs: &Polygon) -> bool {
        arc_polygon_collision(self, rhs)
    }
}

// Point

impl Collides<Point> for Point {
    fn collides(&self, rhs: &Point) -> bool {
        self.is_approximately_equal(rhs)
    }
}

/// Implement `Collides<Arc>` for `Point` by using the standalone function.
impl Collides<Arc> for Point {
    fn collides(&self, rhs: &Arc) -> bool {
        arc_point_collision(rhs, self)
    }
}

impl Collides<Polygon> for Point {
    fn collides(&self, rhs: &Polygon) -> bool {
        rhs.contains(self)
    }
}

// Polygon

impl Collides<Point> for Polygon {
    fn collides(&self, rhs: &Point) -> bool {
        self.contains(rhs)
    }
}

impl Collides<Arc> for Polygon {
    fn collides(&self, rhs: &Arc) -> bool {
        arc_polygon_collision(rhs, self)
    }
}

impl Collides<Polygon> for Polygon {
    fn collides(&self, rhs: &Polygon) -> bool {
        self.outline.iter().any(|point| rhs.contains(point))
    }
}

// ConvecQuadrilateral

impl Collides<Point> for ConvecQuadrilateral {
    fn collides(&self, rhs: &Point) -> bool {
        self.outline.windows(2).all(|outline| {
            let outline = Arc::new(&outline[0], &outline[1]);
            outline.is_on_righthand_side(rhs)
        })
    }
}

impl Collides<ConvecQuadrilateral> for ConvecQuadrilateral {
    fn collides(&self, rhs: &ConvecQuadrilateral) -> bool {
        !self.outline.windows(2).any(|arc| {
            let arc = Arc::new(&arc[0], &arc[1]);
            rhs.outline
                .iter()
                .all(|point| !arc.is_on_righthand_side(point))
        })
    }
}

impl Collides<Arc> for ConvecQuadrilateral {
    fn collides(&self, rhs: &Arc) -> bool {
        self.outline.windows(2).any(|outline| {
            let outline = Arc::new(&outline[0], &outline[1]);
            rhs.collides(&outline)
        })
    }
}

//
// Contains
//

pub trait Contains<Rhs = Self> {
    fn contains(&self, rhs: &Rhs) -> bool;
}

// Polygon

impl Contains<Point> for Polygon {
    fn contains(&self, rhs: &Point) -> bool {
        let north_pole = Point::north_pole();
        let ray = Arc::new(rhs, &north_pole);
        let intersections = self.intersections(&ray).len();
        intersections % 2 == 1
    }
}

impl Contains<Arc> for Polygon {
    fn contains(&self, _rhs: &Arc) -> bool {
        todo!()
    }
}

// ConvecQuadrilateral

impl Contains<Point> for ConvecQuadrilateral {
    fn contains(&self, rhs: &Point) -> bool {
        self.outline
            .windows(2)
            .map(|arc| Arc::new(&arc[0], &arc[1]))
            .all(|arc| arc.is_on_righthand_side(rhs))
    }
}

impl Contains<Arc> for ConvecQuadrilateral {
    fn contains(&self, rhs: &Arc) -> bool {
        self.contains(rhs.from()) && self.contains(rhs.to())
    }
}
