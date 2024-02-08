use std::vec;

use geojson::{Feature, Geometry, Value};

use super::{arc::Arc, point::Point};

#[derive(Clone)]
pub struct Polygon {
    pub outline: Vec<Point>,
}

impl Polygon {
    pub fn new(outline: Vec<Point>) -> Polygon {
        Polygon { outline }
    }

    pub fn arcs(&self) -> Vec<Arc> {
        self.outline
            .windows(2)
            .map(|outline| Arc::new(&outline[0], &outline[1]))
            .collect()
    }

    pub fn intersections(&self, line: &Arc) -> Vec<Point> {
        self.outline
            .windows(2)
            .filter_map(|outline| {
                let outline = Arc::new(&outline[0], &outline[1]);
                line.intersection(&outline)
            })
            .collect()
    }

    pub fn from_geojson_vec(vec: Vec<Vec<f64>>) -> Polygon {
        let outline = vec.into_iter().map(Point::from_geojson_vec).collect();
        Polygon::new(outline)
    }

    pub fn to_geojson_vec(&self) -> Vec<Vec<f64>> {
        self.outline
            .iter()
            .map(|&coordinate| coordinate.to_geojson_vec())
            .collect()
    }

    pub fn to_feature(&self) -> Feature {
        let polygon = self.to_geojson_vec();
        let polygon = Geometry::new(Value::Polygon(vec![polygon]));
        Feature {
            bbox: None,
            geometry: Some(polygon),
            id: None,
            properties: None,
            foreign_members: None,
        }
    }

    pub fn to_json(&self) -> String {
        self.to_feature().to_string()
    }
}
