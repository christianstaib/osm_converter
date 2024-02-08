use geojson::{Feature, Geometry, Value};

use super::{arc::Arc, point::Point};

#[derive(Clone)]
pub struct Linestring {
    pub points: Vec<Point>,
}

impl Linestring {
    pub fn new(points: Vec<Point>) -> Linestring {
        let arcs: Vec<Arc> = points
            .windows(2)
            .map(|arc| Arc::new(&arc[0], &arc[1]))
            // .flatten()
            .collect();
        let mut points: Vec<Point> = arcs.iter().map(|arc| *arc.from()).collect();
        if !points.is_empty() {
            points.push(*points.last().unwrap());
        }

        Linestring { points }
    }

    pub fn to_feature(&self) -> Feature {
        let mut points: Vec<_> = self.points.iter().map(|p| p.to_geojson_vec()).collect();

        if !points.is_empty() {
            let mut factor = 0.0;
            let mut factors = vec![0.0; points.len()];
            factors[0] = 0.0;
            if !points.is_empty() {
                let mut i = 0;
                while i + 1 < points.len() {
                    let pi = &points[i];
                    let pi1 = &points[i + 1];
                    if pi[0].abs() > 170.0 {
                        if (pi[0].signum() == 1.0) && (pi1[0].signum() == -1.0) {
                            factor += 360.0;
                        } else if (pi[0].signum() == -1.0) && (pi1[0].signum() == 1.0) {
                            factor -= 360.0;
                        }
                    }

                    i += 1;
                    factors[i] = factor;
                }
            }

            for i in 0..points.len() {
                points[i][0] += factors[i];
            }
        }

        let points = Geometry::new(Value::LineString(points));
        Feature {
            bbox: None,
            geometry: Some(points),
            id: None,
            properties: None,
            foreign_members: None,
        }
    }
}
