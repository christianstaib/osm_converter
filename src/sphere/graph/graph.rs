use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    usize,
};

use indicatif::ProgressIterator;

use crate::sphere::geometry::{
    arc::Arc,
    planet::Planet,
    point::{radians_to_meter, Point},
};

pub struct Fmi {
    pub points: Vec<Point>,
    pub arcs: Vec<Arc>,
}

impl Fmi {
    pub fn from_gr_co_file(&self, gr_path: &str, co_path: &str) -> Fmi {
        let mut arcs = Vec::new();
        let mut points = HashMap::new();

        //
        let co_reader = BufReader::new(File::open(co_path).unwrap());
        let co_lines = co_reader.lines();

        co_lines.for_each(|line| {
            let line = line.unwrap();
            let line_sections: Vec<_> = line.split_whitespace().into_iter().collect();
            if let Some(first_line_section) = line_sections.first() {
                if first_line_section == &"v" {
                    let id: u32 = line_sections.get(1).unwrap().parse().unwrap();
                    let lat: f64 = line_sections.get(2).unwrap().parse().unwrap();
                    let lon: f64 = line_sections.get(3).unwrap().parse().unwrap();
                    points.insert(id, Point::from_coordinate(lat, lon));
                }
            }
        });

        //
        let gr_reader = BufReader::new(File::open(gr_path).unwrap());
        let gr_lines = gr_reader.lines();

        gr_lines.for_each(|line| {
            let line = line.unwrap();
            let line_sections: Vec<_> = line.split_whitespace().into_iter().collect();
            if let Some(first_line_section) = line_sections.first() {
                if first_line_section == &"a" {
                    let tail: u32 = line_sections.get(1).unwrap().parse().unwrap();
                    let head: u32 = line_sections.get(2).unwrap().parse().unwrap();
                    let _weight: u32 = line_sections.get(3).unwrap().parse().unwrap();
                    arcs.push(Arc::new(
                        &points.get(&tail).unwrap(),
                        &points.get(&head).unwrap(),
                    ));
                }
            }
        });

        let points = points.into_iter().map(|(_, point)| point).collect();

        Fmi { points, arcs }
    }

    pub fn to_gr_co_file(&self, gr_path: &str, co_path: &str) {
        println!("writing to file");
        let mut point_id_map = HashMap::new();
        for (i, point) in self.points.iter().enumerate() {
            point_id_map.insert(point, i);
        }

        let mut arc_map: HashMap<(u32, u32), u32> = HashMap::new();
        self.arcs.iter().for_each(|arc| {
            // srcIDX trgIDX cost type maxspeed
            let source = *point_id_map.get(arc.from()).unwrap() as u32;
            let target = *point_id_map.get(arc.to()).unwrap() as u32;
            let cost = radians_to_meter(arc.central_angle()).round() as u32;
            arc_map.insert((source, target), cost);
            arc_map.insert((target, source), cost);
        });

        // write arcs
        let mut gr_writer = BufWriter::new(File::create(gr_path).unwrap());
        writeln!(gr_writer, "p sp {} {}", self.points.len(), self.arcs.len()).unwrap();
        arc_map.iter().for_each(|((tail, head), weight)| {
            writeln!(gr_writer, "a {} {} {}", tail, head, weight).unwrap();
        });
        gr_writer.flush().unwrap();

        // write points
        let mut co_writer = BufWriter::new(File::create(co_path).unwrap());
        writeln!(co_writer, "p aux sp co {}", self.points.len(),).unwrap();
        self.points.iter().for_each(|point| {
            // nodeID nodeID2 latitude longitude elevation
            writeln!(
                co_writer,
                "v {} {} {}",
                point_id_map.get(point).unwrap(),
                point.latitude(),
                point.longitude()
            )
            .unwrap();
        });
        co_writer.flush().unwrap();
    }

    pub fn to_file(&self, path: &str) {
        println!("writing to file");
        let mut point_id_map = HashMap::new();
        for (i, point) in self.points.iter().enumerate() {
            point_id_map.insert(point, i);
        }

        let mut arc_map: HashMap<(u32, u32), u32> = HashMap::new();
        self.arcs.iter().for_each(|arc| {
            // srcIDX trgIDX cost type maxspeed
            let source = *point_id_map.get(arc.from()).unwrap() as u32;
            let target = *point_id_map.get(arc.to()).unwrap() as u32;
            let cost = radians_to_meter(arc.central_angle()).round() as u32;
            arc_map.insert((source, target), cost);
            arc_map.insert((target, source), cost);
        });

        let mut writer = BufWriter::new(File::create(path).unwrap());
        writeln!(writer, "{}", self.points.len()).unwrap();
        writeln!(writer, "{}", arc_map.len()).unwrap();
        self.points.iter().for_each(|point| {
            // nodeID nodeID2 latitude longitude elevation
            writeln!(
                writer,
                "{} 0 {} {} 0",
                point_id_map.get(point).unwrap(),
                point.latitude(),
                point.longitude()
            )
            .unwrap();
        });
        writer.flush().unwrap();

        arc_map.iter().for_each(|((source, target), cost)| {
            // srcIDX trgIDX cost type maxspeed
            writeln!(writer, "{} {} {} 0 0", source, target, cost).unwrap();
        });
        writer.flush().unwrap();
    }

    pub fn to_planet(&self) -> Planet {
        let mut planet = Planet::new();
        planet.arcs = self.arcs.clone();
        planet
    }

    pub fn nearest(&self, lon: f64, lat: f64) -> u32 {
        let point = Point::from_coordinate(lat, lon);
        self.points
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let a = Arc::new(a, &point);
                let b = Arc::new(b, &point);
                a.central_angle().partial_cmp(&b.central_angle()).unwrap()
            })
            .map(|(i, _)| i)
            .unwrap()
            .try_into()
            .unwrap()
    }

    pub fn id_to_point(&self, id: u32) -> Point {
        self.points[id as usize]
    }

    pub fn convert_path(&self, path: &Vec<u32>) -> Vec<Point> {
        path.iter().map(|&id| self.points[id as usize]).collect()
    }
}
