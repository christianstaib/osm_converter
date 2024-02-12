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
    pub fn from_file(path: &str) -> Fmi {
        let reader = BufReader::new(File::open(path).unwrap());
        let mut lines = reader.lines();

        let num_nodes = lines.by_ref().next().unwrap().unwrap().parse().unwrap();
        let num_arcs = lines.by_ref().next().unwrap().unwrap().parse().unwrap();

        let points: Vec<_> = lines
            .by_ref()
            .take(num_nodes)
            .enumerate()
            .progress_count(num_nodes as u64)
            .map(|(i, line)| {
                // nodeID nodeID2 latitude longitude elevation
                let line = line.unwrap();
                let mut line = line.split_whitespace();
                let id: u32 = line.next().unwrap().parse().unwrap();
                assert_eq!(id as usize, i);
                let lat = line.next().unwrap().parse().unwrap();
                let lon = line.next().unwrap().parse().unwrap();
                Point::from_coordinate(lat, lon)
            })
            .collect();

        let arcs = lines
            .by_ref()
            .take(num_arcs)
            .progress_count(num_arcs as u64)
            .map(|line| {
                let line = line.unwrap();
                let mut line = line.split_whitespace();
                let from: u32 = line.next().unwrap().parse().unwrap();
                let to: u32 = line.next().unwrap().parse().unwrap();
                let _cost: u32 = line.next().unwrap().parse().unwrap();
                Arc::new(&points[from as usize], &points[to as usize])
            })
            .collect();

        Fmi { points, arcs }
    }

    pub fn to_gr_file(&self, path: &str) {
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
        writeln!(writer, "p sp {} {}", self.points.len(), self.arcs.len()).unwrap();
        // writeln!(writer, "{}", arc_map.len()).unwrap();
        // self.points.iter().for_each(|point| {
        //     // nodeID nodeID2 latitude longitude elevation
        //     writeln!(
        //         writer,
        //         "{} 0 {} {} 0",
        //         point_id_map.get(point).unwrap(),
        //         point.latitude(),
        //         point.longitude()
        //     )
        //     .unwrap();
        // });
        // writer.flush().unwrap();

        arc_map.iter().for_each(|((tail, head), weight)| {
            // srcIDX trgIDX cost type maxspeed
            writeln!(writer, "a {} {} {} 0 0", tail, head, weight).unwrap();
        });
        writer.flush().unwrap();
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
