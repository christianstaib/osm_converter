use std::f64::consts::PI;

use geojson::{Feature, Geometry, Value};
use nalgebra::Vector3;
use rand::Rng;

use super::point::Point;

/// Represents a minor arc, e.g. the shortest path between to points, called 'from' and 'to'.
#[derive(Clone, PartialEq)]
pub struct Arc {
    from: Point,
    to: Point,
}

impl Arc {
    pub fn new(start: &Point, end: &Point) -> Arc {
        Arc {
            from: *start,
            to: *end,
        }
    }

    // Returns the intial bearing from 'from' to 'to' in radians.
    pub fn initial_bearing(&self) -> f64 {
        let north_pole = Point::north_pole();

        let from_to_normal = self.from.n_vector().cross(self.to.n_vector());
        let from_north_pole_normal = self.from.n_vector().cross(north_pole.n_vector());

        let mut sign = 1.0;
        if from_to_normal
            .cross(&from_north_pole_normal)
            .dot(self.from.n_vector())
            < 0.0
        {
            sign = -1.0;
        }

        let sine_theta = from_to_normal.cross(&from_north_pole_normal).magnitude() * sign;
        let cosine_theta = from_to_normal.dot(&from_north_pole_normal);

        sine_theta.atan2(cosine_theta).rem_euclid(2.0 * PI)
    }

    /// Returns true if point is on the right hand side, looking from 'from' to 'to'.
    pub fn is_on_righthand_side(&self, point: &Point) -> bool {
        self.normal().dot(point.n_vector()) > 0.0
    }

    /// Returns a point that is somwhere on the arc.
    pub fn random_intermediate_point(&self) -> Point {
        let mut rng = rand::thread_rng();
        let f = rng.gen_range(0.0..1.0);
        Point::from_n_vector(&((self.from.n_vector() * (1.0 - f) + self.to.n_vector() * f) / 2.0))
    }

    /// Returns the midpoint of the arc.
    pub fn middle(&self) -> Point {
        Point::from_n_vector(&((self.from.n_vector() + self.to.n_vector()) / 2.0))
    }

    /// Returns the 'from' point of the arc, e.g. the starting point.
    pub fn from(&self) -> &Point {
        &self.from
    }

    /// Returns the 'to' point of the arc, e.g. the ending point.
    pub fn to(&self) -> &Point {
        &self.to
    }

    /// Returns the intersection point between self and other, if it exists.
    ///
    /// Caveate: If the intersection point is near self.from it will not be returned. If it is near
    /// self.to it will be returned. This ensures that for a continoues path of arcs only one
    /// intersection is returned. So this functions is not symetrical. If you want to check for
    /// path, you need to ensure that for all arcs in the path you call arc.intersection(ray).
    pub fn intersection(&self, other: &Arc) -> Option<Point> {
        if self.to.is_approximately_equal(&other.from) || self.to.is_approximately_equal(&other.to)
        {
            return Some(self.to);
        }

        // check if intersection of both great circles lies on both arcs
        let candidate = self.normal().cross(&other.normal()).normalize();
        if !candidate.x.is_nan() && !candidate.y.is_nan() && !candidate.z.is_nan() {
            let candidate = Point::from_n_vector(&candidate);
            if self.between_normals(&candidate)
                && other.between_normals(&candidate)
                && !candidate.is_approximately_equal(self.from())
            {
                return Some(candidate);
            }
            let candidate = candidate.antipode();
            if self.between_normals(&candidate)
                && other.between_normals(&candidate)
                && !candidate.is_approximately_equal(self.from())
            {
                return Some(candidate);
            }
        }

        None
    }

    /// Checks if self intersects other.
    ///
    /// Caveate: If the intersection point is near self.from, return false . If it is near
    /// self.to, return true. This ensures that for a continoues path of arcs only one
    /// intersection is returned. So this functions is not symetrical. If you want to check for
    /// path, you need to ensure that for all arcs in the path you call arc.intersection(ray).
    pub fn intersects(&self, other: &Arc) -> bool {
        self.intersection(other).is_some()
    }

    /// Calculates the normal vector of the the arc. The normal vector defines a plane thrugh zero,
    /// 'from' and 'to'.
    pub fn normal(&self) -> Vector3<f64> {
        self.from.n_vector().cross(self.to.n_vector()).normalize()
    }

    /// Calculates an vector that is perpendicular to the normal and 'from'.
    fn from_normal(&self) -> Vector3<f64> {
        self.normal().cross(self.from.n_vector()).normalize()
    }

    /// Calculates an vector that is perpendicular to the normal and 'to'.
    fn to_normal(&self) -> Vector3<f64> {
        self.normal().cross(self.to.n_vector()).normalize()
    }

    /// Returns true if point lies between from_normal and to_normal.
    fn between_normals(&self, point: &Point) -> bool {
        let a0 = point.n_vector().dot(&self.from_normal());
        let a1 = point.n_vector().dot(&self.to_normal());

        a0 >= 0.0 && a1 <= 0.0
    }

    /// Returns the central angle of the arc in radians.
    pub fn central_angle(&self) -> f64 {
        let from = self.from.n_vector();
        let to = self.to.n_vector();
        from.angle(to)
    }

    /// Creates an arc from a GeoJSON-compatible vector. Note the GeoJSON order, which is longitude first.
    pub fn from_geojson_vec(vec: Vec<Vec<f64>>) -> Arc {
        Arc::new(
            &Point::from_geojson_vec(vec[0].clone()),
            &Point::from_geojson_vec(vec[1].clone()),
        )
    }

    /// Creates a GeoJSON-compatible vector representing the arc. Note the GeoJSON order, which is longitude first.
    pub fn to_geojson_vec(&self) -> Vec<Vec<f64>> {
        vec![self.from.to_geojson_vec(), self.to.to_geojson_vec()]
    }

    pub fn to_feature(&self) -> Feature {
        let point = Geometry::new(Value::LineString(vec![
            self.from.to_geojson_vec(),
            self.to.to_geojson_vec(),
        ]));
        Feature {
            bbox: None,
            geometry: Some(point),
            id: None,
            properties: None,
            foreign_members: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use crate::sphere::geometry::{arc::Arc, point::Point};

    #[test]
    fn test_central_angle1() {
        let from = Point::from_coordinate(90.0, 0.0);
        let to = Point::from_coordinate(0.0, 0.0);
        let arc = Arc::new(&from, &to);
        let angle = arc.central_angle();
        assert!((angle - (PI / 2.0)).abs() < 1e-10, "angle was {}", angle);
    }

    #[test]
    fn test_central_angle2() {
        let from = Point::from_coordinate(0.0, 135.0);
        let to = Point::from_coordinate(0.0, -135.0);
        let arc = Arc::new(&from, &to);
        let angle = arc.central_angle();
        assert!((angle - (PI / 2.0)).abs() < 1e-10, "angle was {}", angle);
    }

    #[test]
    fn test_central_angle3() {
        let from = Point::from_coordinate(0.0, 135.0);
        let to = Point::from_coordinate(0.0, -135.0);
        let arc = Arc::new(&from, &to);
        let angle = arc.central_angle();
        assert!((angle - (PI / 2.0)).abs() < 1e-10, "angle was {}", angle);
    }

    #[test]
    fn test_intersection() {
        let outline_from = Point::from_coordinate(10.9602021, 119.7085977);
        let outline_to = Point::from_coordinate(10.9380527, 119.7102928);
        let outline = Arc::new(&outline_from, &outline_to);

        let ray_from = Point::from_coordinate(10.939165355971703, 119.71220924280686);
        let ray_to = Point::from_coordinate(11.42324706114331, 119.42008985034511);
        let ray = Arc::new(&ray_from, &ray_to);

        let intersect = ray.intersects(&outline);
        assert!(intersect)
    }

    #[test]
    fn test_edge_intersection() {
        let outline = [
            Point::from_coordinate(-1.0, 0.0),
            Point::from_coordinate(0.0, 0.0),
            Point::from_coordinate(1.0, 0.0),
        ];

        let ray = Arc::new(
            &Point::from_coordinate(0.0, 1.0),
            &Point::from_coordinate(0.0, -1.0),
        );

        let arc0 = Arc::new(&outline[0], &outline[1]);
        let arc1 = Arc::new(&outline[1], &outline[2]);

        assert!(arc0.intersects(&ray));
        assert!(!arc1.intersects(&ray));
    }
}
