use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    str::FromStr,
};

use geojson::{Feature, Value};
use image::{GrayImage, Luma};
use imageproc::{drawing::draw_antialiased_line_segment_mut, pixelops::interpolate};
use indicatif::ProgressIterator;

use super::{
    arc::Arc,
    collision_detection::{CollisionDetection, Contains},
    linestring::Linestring,
    osm_data::OsmData,
    point::Point,
    polygon::Polygon,
};

#[derive(Clone)]
pub struct Planet {
    pub polygons: Vec<Polygon>,
    pub points: Vec<Point>,
    pub arcs: Vec<Arc>,
    pub linestrings: Vec<Linestring>,
}

impl CollisionDetection for Planet {
    fn is_on_polygon(&self, point: &Point) -> bool {
        self.polygons.iter().any(|polygon| polygon.contains(point))
    }
}

impl Default for Planet {
    fn default() -> Self {
        Self::new()
    }
}

impl Planet {
    /// Creates a new, empty planet.
    pub fn new() -> Self {
        Self {
            polygons: Vec::new(),
            points: Vec::new(),
            arcs: Vec::new(),
            linestrings: Vec::new(),
        }
    }

    /// Returns the intersection points between all polygons and the arc.
    pub fn intersections(&self, arc: &Arc) -> Vec<Point> {
        self.polygons
            .iter()
            .flat_map(|polygon| polygon.intersections(arc))
            .collect()
    }

    pub fn from_osm_file(path: &str) -> Self {
        let raw_osm_data = OsmData::from_path(path);
        raw_osm_data.to_planet()
    }

    pub fn to_image(&self, path: &str) {
        let white = Luma([255u8]);

        // lon, lat
        let min: (f64, f64) = (-180.0, -90.0);
        let max: (f64, f64) = (180.0, 90.0);

        let pix_per_unit = 100.0;
        let x_pix = ((max.0 - min.0) * pix_per_unit) as u32;
        let y_pix = ((max.1 - min.1) * pix_per_unit) as u32;
        let mut image = GrayImage::new(x_pix, y_pix);

        for arc in self.arcs.iter().progress() {
            let start = arc.from().to_geojson_vec();
            let mut end = arc.to().to_geojson_vec();

            if start[0] < -170.0 && end[0] > 170.0 {
                end[0] -= 360.0;
            } else if start[0] > 170.0 && end[0] < -170.0 {
                end[0] += 360.0;
            }

            let start = (
                scale(start[0], min.0, max.0, 0, x_pix),
                scale(-start[1], min.1, max.1, 0, y_pix),
            );
            let end = (
                scale(end[0], min.0, max.0, 0, x_pix),
                scale(-end[1], min.1, max.1, 0, y_pix),
            );

            draw_antialiased_line_segment_mut(&mut image, start, end, white, interpolate);
        }

        // Save the image
        image.save(path).unwrap();
    }

    pub fn from_geojson_file(path: &str) -> Result<Planet, Box<dyn Error>> {
        let reader = BufReader::new(File::open(path).unwrap());
        let mut planet = Planet::new();

        reader
            .lines()
            .filter_map(|line| {
                let mut line = line.unwrap();
                if line.ends_with(',') {
                    line.pop();
                }
                Feature::from_str(line.as_str()).ok()?.geometry
            })
            .for_each(|geometry| match geometry.value {
                Value::Point(point) => planet.points.push(Point::from_geojson_vec(point)),
                Value::Polygon(polygon) => planet
                    .polygons
                    .push(Polygon::from_geojson_vec(polygon[0].clone())),
                Value::LineString(line) => planet.arcs.push(Arc::from_geojson_vec(line)),
                _ => (),
            });

        Ok(planet)
    }

    pub fn to_geojson_str(&self) -> String {
        let mut features = Vec::new();
        features.extend(self.points.iter().map(|point| point.to_feature()));
        features.extend(self.polygons.iter().map(|polygon| polygon.to_feature()));
        features.extend(self.arcs.iter().map(|line| line.to_feature()));
        features.extend(
            self.linestrings
                .iter()
                .map(|linestring| linestring.to_feature()),
        );

        let mut writer = String::new();
        writer += r#"{"type":"FeatureCollection","features":["#;

        let mut features = features.into_iter().peekable();
        while let Some(feature) = features.next() {
            if features.peek().is_some() {
                writer += &format!("{},", feature);
            } else {
                writer += &format!("{}", feature);
            }
        }
        writer += r#"]}"#;
        writer
    }

    pub fn to_geojson_file(&self, path: &str) {
        println!("writing to file");
        let mut features = Vec::new();
        features.extend(self.points.iter().map(|point| point.to_feature()));
        features.extend(self.polygons.iter().map(|polygon| polygon.to_feature()));
        features.extend(self.arcs.iter().map(|line| line.to_feature()));
        features.extend(
            self.linestrings
                .iter()
                .map(|linestring| linestring.to_feature()),
        );

        let mut writer = BufWriter::new(File::create(path).unwrap());
        writeln!(writer, r#"{{"type":"FeatureCollection","features":["#,).unwrap();

        let mut features = features.into_iter().peekable();
        while let Some(feature) = features.next() {
            if features.peek().is_some() {
                writeln!(writer, "{},", feature).unwrap();
            } else {
                writeln!(writer, "{}", feature).unwrap();
            }
        }
        writeln!(writer, r#"]}}"#,).unwrap();
        writer.flush().unwrap();
    }
}

pub fn scale(input: f64, input_min: f64, input_max: f64, output_min: u32, output_max: u32) -> i32 {
    let input_range = input_max - input_min;
    let output_range = output_max as f64 - output_min as f64;
    let scaled_value = ((input - input_min) / input_range) * output_range;
    (scaled_value + output_min as f64).round() as i32
}
