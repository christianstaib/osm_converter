use std::collections::HashMap;

use indicatif::ProgressBar;
use osmpbf::{Element, ElementReader};

use super::{planet::Planet, point::Point, polygon::Polygon};

/// a planet struct which ways are not cloesed
pub struct OsmData {
    pub nodes: HashMap<i64, Point>,
    pub coastlines: Vec<Vec<i64>>,
}

impl OsmData {
    pub fn to_planet(&self) -> Planet {
        let mut planet = Planet::new();
        planet.polygons.extend(
            self.coastlines
                .iter()
                .map(|coastline| {
                    coastline
                        .iter()
                        .map(|node_id| self.nodes[&node_id])
                        .collect()
                })
                .map(Polygon::new),
        );

        planet
    }

    pub fn from_path(path: &str) -> OsmData {
        let mut nodes = HashMap::new();
        let mut coastlines = Vec::new();

        let reader = ElementReader::from_path(path).unwrap();
        let elements = reader
            .par_map_reduce(|_| 1, || 0_u64, |a, b| a + b)
            .unwrap();
        println!("there are {}", elements);

        let pb = ProgressBar::new(elements);
        let reader = ElementReader::from_path(path).unwrap();
        reader
            .for_each(|element| {
                pb.inc(1);
                match element {
                    Element::DenseNode(node) => {
                        nodes.insert(node.id(), Point::from_coordinate(node.lat(), node.lon()));
                    }
                    Element::Way(way) => {
                        if way
                            .tags()
                            .any(|(key, value)| key == "natural" && value == "coastline")
                        {
                            coastlines.push(way.refs().collect());
                        }
                    }
                    _ => (),
                }
            })
            .unwrap();
        pb.finish();
        println!("finished reading");

        let mut raw_osm_data = OsmData { nodes, coastlines };
        raw_osm_data.close_coastline();
        raw_osm_data
    }

    fn close_coastline(&mut self) {
        println!("closing coastlines");
        let (mut open_coastlines, mut closed_coastlines): (Vec<_>, Vec<_>) = self
            .coastlines
            .drain(..)
            .partition(|coastline| coastline.first().unwrap() != coastline.last().unwrap());

        open_coastlines.sort_unstable_by_key(|coastline| coastline.first().cloned());

        let pb = ProgressBar::new(open_coastlines.len() as u64);
        while let Some(mut coastline) = open_coastlines.pop() {
            pb.inc(1);
            let mut last = *coastline.last().unwrap();
            while let Ok(to_append) = open_coastlines
                .binary_search_by_key(&last, |other_coastline| *other_coastline.first().unwrap())
            {
                let mut to_append = open_coastlines.remove(to_append);
                coastline.append(&mut to_append);
                last = *coastline.last().unwrap();
            }
            coastline.dedup();
            closed_coastlines.push(coastline);
        }
        pb.finish();
        self.coastlines = closed_coastlines;
    }
}
