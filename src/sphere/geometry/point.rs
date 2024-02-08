use std::{f64::consts::PI, fmt, hash::Hash};

use geojson::{Feature, Geometry, Value};
use nalgebra::Vector3;
use rand::Rng;

use super::arc::Arc;

/// Represents a point on the Earth's surface using an n-vector, which is a normalised vector
/// perpendicular to the Earth's surface.
#[derive(Clone, Copy, PartialEq)]
pub struct Point {
    n_vector: Vector3<f64>,
}

impl Hash for Point {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.n_vector.x.to_bits().hash(state);
        self.n_vector.y.to_bits().hash(state);
        self.n_vector.z.to_bits().hash(state);
    }
}

impl Eq for Point {}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(lat:{}, lon::{})", self.latitude(), self.longitude())
    }
}

pub struct PointGenerator {}

impl Iterator for PointGenerator {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Point::random())
    }
}

impl Default for PointGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl PointGenerator {
    pub fn new() -> PointGenerator {
        PointGenerator {}
    }
}

impl Point {
    /// Creates a `Point` from given latitude and longitude values, asserting that they are within valid ranges
    /// (-90.0 <= latitude <= 90, -180.0 <= longitude <= 180.0).
    pub fn from_coordinate(latitude: f64, longitude: f64) -> Point {
        assert!(
            (-90.0..=90.0).contains(&latitude),
            "illegal lat: {}",
            latitude
        );
        assert!(
            (-180.0..=180.0).contains(&longitude),
            "illegal lon: {}",
            longitude
        );

        let lat_rad = latitude.to_radians();
        let lon_rad = longitude.to_radians();
        let n_vector = Vector3::new(
            lat_rad.cos() * lon_rad.cos(),
            lat_rad.cos() * lon_rad.sin(),
            lat_rad.sin(),
        );

        Point { n_vector }
    }

    /// Constructs a point from a n-vector.
    pub fn from_n_vector(n_vector: &Vector3<f64>) -> Point {
        let point = Point {
            n_vector: *n_vector,
        };

        let lat = point.latitude();
        let lon = point.longitude();
        assert!((-90.0..=90.0).contains(&lat), "illegal vec: {}", n_vector);
        assert!((-180.0..=180.0).contains(&lon), "illegal vec: {}", n_vector);

        point
    }

    /// Returns a point representing the geographic north pole.
    pub fn north_pole() -> Point {
        Point::from_coordinate(90.0, 0.0)
    }

    /// Returns a point representing the geographic south pole.
    pub fn south_pole() -> Point {
        Point::from_coordinate(-90.0, 0.0)
    }

    pub fn random() -> Point {
        let mut rng = rand::thread_rng();
        let y: f64 = rng.gen_range(-1.0..1.0);
        let lat_rad: f64 = y.asin();
        let lat: f64 = lat_rad.to_degrees();
        let lon: f64 = rng.gen_range(-180.0..180.0);
        Point::from_coordinate(lat, lon)
    }

    pub fn random_generator() -> impl Iterator<Item = Point> {
        std::iter::repeat_with(Point::random)
    }

    /// Calculates the destination point given a start point, bearing, and distance on a sphere.
    /// The destination is computed using vector math on a unit sphere where:
    /// - `start`: the n-vector representing the start point
    /// - `bearing_rad`: the bearing (from north) in radians
    /// - `distance_rad`: the angular distance travelled in radians
    pub fn destination_point(start: &Point, bearing_rad: f64, distance_rad: f64) -> Point {
        let north_pole = Point::north_pole();
        let east_direction = north_pole.n_vector().cross(start.n_vector()).normalize();
        let north_direction = start.n_vector().cross(&east_direction);
        let direction = north_direction * bearing_rad.cos() + east_direction * bearing_rad.sin();
        let destination = start.n_vector() * distance_rad.cos() + direction * distance_rad.sin();
        Point::from_n_vector(&destination)
    }

    /// Returns the latitude of the point in degrees. It is calculated each time it is called.
    pub fn latitude(&self) -> f64 {
        self.n_vector
            .z
            .atan2((self.n_vector.x.powi(2) + self.n_vector.y.powi(2)).sqrt())
            .to_degrees()
    }

    /// Returns the longitude of the point in degrees. It is calculated each time it is called.
    pub fn longitude(&self) -> f64 {
        self.n_vector
            .y
            .to_radians()
            .atan2(self.n_vector.x.to_radians())
            .to_degrees()
    }

    /// Returns the n-vector of the point.
    pub fn n_vector(&self) -> &Vector3<f64> {
        &self.n_vector
    }

    /// Determines if two points are approximately equal within 0.1 meters tolerance.
    /// Returns `true` if the points are within this tolerance, otherwise `false`.
    pub fn is_approximately_equal(&self, other: &Point) -> bool {
        Arc::new(self, other).central_angle() <= meters_to_radians(0.25)
    }

    /// Returns the the antipodal point to self.
    pub fn antipode(&self) -> Point {
        Point::from_n_vector(&-self.n_vector)
    }

    /// Converts the point to a GeoJSON-compatible [longitude, latitude] vector.
    /// The order is longitude first to comply with the GeoJSON specification.
    pub fn to_geojson_vec(&self) -> Vec<f64> {
        vec![self.longitude(), self.latitude()]
    }

    /// Creates a point from a GeoJSON-compatible [longitude, latitude] vector.
    /// Note the GeoJSON order, which is longitude first.
    pub fn from_geojson_vec(vec: Vec<f64>) -> Point {
        Point::from_coordinate(vec[1], vec[0])
    }

    pub fn to_feature(&self) -> Feature {
        let point: Vec<f64> = self.to_geojson_vec();
        let point = Geometry::new(Value::Point(point));
        Feature {
            bbox: None,
            geometry: Some(point),
            id: None,
            properties: None,
            foreign_members: None,
        }
    }
}

pub fn meters_to_radians(meters: f64) -> f64 {
    const EARTH_CIRCUMFERENCE_METERS: f64 = 40_000_000.0;
    meters * ((2.0 * PI) / EARTH_CIRCUMFERENCE_METERS)
}

pub fn radians_to_meter(radians: f64) -> f64 {
    const EARTH_CIRCUMFERENCE_METERS: f64 = 40_000_000.0;
    radians * (EARTH_CIRCUMFERENCE_METERS / (2.0 * PI))
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use crate::sphere::geometry::point::{meters_to_radians, radians_to_meter, Point};

    #[test]
    fn conversion_between_n_vector_and_coordinates() {
        for _ in 0..100 {
            let point = Point::random();
            assert!(point.is_approximately_equal(&Point::from_n_vector(point.n_vector())));
        }
        for _ in 0..100 {
            let point = Point::random();
            assert!(point.is_approximately_equal(&Point::from_coordinate(
                point.latitude(),
                point.longitude()
            )));
        }
    }

    #[test]
    fn meters_to_radians_test() {
        let m = 10_000_000.0; // should be arround 1/4 of earths circumference
        let rad = meters_to_radians(m);
        assert!((rad - (PI / 2.0)).abs() < 0.01, "{}", rad);
    }

    #[test]
    fn radians_to_meter_test() {
        let rad = PI / 2.0;
        let m = radians_to_meter(rad);
        assert!((m - 10_000_000.0).abs() < 0.01, "{}", m);
    }
}
