use indicatif::{ProgressBar, ProgressIterator};
use osm_test::radians_to_meter;
use osm_test::{
    meters_to_radians, Arc, CollisionDetection, Planet, PlanetGrid, Point, PointPlanetGrid, Polygon,
};
use rayon::prelude::{ParallelBridge, ParallelIterator};

use std::collections::HashMap;
use std::io::BufWriter;
use std::io::Write;
use std::{env, f64::consts::PI, fs::File};

const SEARCH_RADIUS: f64 = 35_000.0;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    const PLANET_PATH: &str = "tests/data/geojson/planet.geojson";
    let in_planet = Planet::from_geojson_file(PLANET_PATH).unwrap();
    // const PLANET_PATH: &str = "tests/data/osm/planet-coastlines.osm.pbf";
    // let in_planet = Planet::from_osm_file(PLANET_PATH);

    const OUT_PLANET_PATH: &str = "tests/data/test_geojson/network.geojson";

    let planet_grid = generate_planet_grid(&in_planet);
    let points = generate_points(4_000_000, &planet_grid);
    let point_grid = generate_point_grid(&points);
    let arcs = generate_arcs(&points, &point_grid, &planet_grid);

    let mut out_planet = Planet::new();
    out_planet.arcs = arcs
        .iter()
        .map(|arc| arc._make_good_line())
        .flatten()
        .collect();
    out_planet.to_geojson_file(OUT_PLANET_PATH);

    arcs_to_file(&arcs, &points);
}

fn generate_points(how_many: u32, planet_grid: &PlanetGrid) -> Vec<Point> {
    println!("generating points");
    let mut points = Vec::new();

    let pb = ProgressBar::new(how_many as u64);
    while points.len() < how_many as usize {
        let mut point = Point::random();
        if !planet_grid.is_on_polygon(&point) {
            point.id = Some(points.len() as u32);
            points.push(point);
            pb.inc(1);
        }
    }
    pb.finish_and_clear();

    points
}

fn generate_point_grid(points: &Vec<Point>) -> PointPlanetGrid {
    println!("generating point grid");
    let mut point_grid = PointPlanetGrid::new(10);
    points
        .iter()
        .progress()
        .for_each(|point| point_grid.add_point(point));
    point_grid
}

fn generate_planet_grid(planet: &Planet) -> PlanetGrid {
    println!("generating planet grid");
    let mut planet_grid = PlanetGrid::new(50);
    planet
        .polygons
        .iter()
        .progress()
        .for_each(|polygon| planet_grid.add_polygon(polygon));
    planet_grid.update_midpoints();
    planet_grid
}

fn arcs_to_file(arcs: &Vec<Arc>, points: &Vec<Point>) {
    let mut writer = BufWriter::new(File::create("test.fmi").unwrap());
    writeln!(writer, "{}", points.len()).unwrap();
    writeln!(writer, "{}", arcs.len()).unwrap();
    println!("writing points to file");
    points.iter().progress().for_each(|point| {
        writeln!(
            writer,
            "{} {} {}",
            point.id.unwrap(),
            point.latitude(),
            point.longitude()
        )
        .unwrap();
    });
    writer.flush().unwrap();

    println!("writing arcs to file");
    arcs.iter().progress().for_each(|arc| {
        writeln!(
            writer,
            "{} {} {}",
            arc.from().id.unwrap(),
            arc.to().id.unwrap(),
            (radians_to_meter(arc.central_angle()) * 1.0) as u32
        )
        .unwrap();
    });
    writer.flush().unwrap();
}

fn generate_arcs(
    points: &Vec<Point>,
    point_grid: &PointPlanetGrid,
    planet_grid: &PlanetGrid,
) -> Vec<Arc> {
    println!("generating arcs");
    let arcs: Vec<_> = points
        .iter()
        .progress()
        .par_bridge()
        .map(|point| {
            vec![ur(point), lr(point), ll(point), ul(point)]
                .iter()
                .filter_map(|polygon| {
                    let mut local_points = point_grid.get_points(&polygon);
                    local_points.sort_unstable_by(|x, y| {
                        Arc::new(point, x)
                            .central_angle()
                            .total_cmp(&Arc::new(point, y).central_angle())
                    });

                    // .get(1) is point
                    if let Some(target) = local_points.get(1) {
                        return Some(Arc::new(point, &target));
                    }

                    None
                })
                //.filter(|arc| !planet_grid.check_collision(arc))
                // .map(|arc| vec![arc, Arc::new(arc.to(), arc.from())])
                // .flatten()
                .collect::<Vec<_>>()
        })
        .flatten()
        .collect();

    let mut hash_map = HashMap::new();
    for arc in arcs.into_iter().progress() {
        hash_map.insert((arc.from().id.unwrap(), arc.to().id.unwrap()), arc);
    }

    hash_map.into_values().collect()
}

// works

fn ur(point: &Point) -> Polygon {
    let cloned_point = point.clone();
    Polygon::new(vec![
        cloned_point,
        Point::destination_point(&point, 2.0 / 4.0 * PI, meters_to_radians(SEARCH_RADIUS)),
        Point::destination_point(&point, 1.0 / 4.0 * PI, meters_to_radians(SEARCH_RADIUS)),
        Point::destination_point(&point, 0.0 / 4.0 * PI, meters_to_radians(SEARCH_RADIUS)),
        cloned_point,
    ])
}

// works
fn lr(point: &Point) -> Polygon {
    let cloned_point = point.clone();
    Polygon::new(vec![
        cloned_point,
        Point::destination_point(&point, 4.0 / 4.0 * PI, meters_to_radians(SEARCH_RADIUS)),
        Point::destination_point(&point, 3.0 / 4.0 * PI, meters_to_radians(SEARCH_RADIUS)),
        Point::destination_point(&point, 2.0 / 4.0 * PI, meters_to_radians(SEARCH_RADIUS)),
        cloned_point,
    ])
}
fn ll(point: &Point) -> Polygon {
    let cloned_point = point.clone();
    Polygon::new(vec![
        cloned_point,
        Point::destination_point(&point, 6.0 / 4.0 * PI, meters_to_radians(SEARCH_RADIUS)),
        Point::destination_point(&point, 5.0 / 4.0 * PI, meters_to_radians(SEARCH_RADIUS)),
        Point::destination_point(&point, 4.0 / 4.0 * PI, meters_to_radians(SEARCH_RADIUS)),
        cloned_point,
    ])
}
fn ul(point: &Point) -> Polygon {
    let cloned_point = point.clone();
    Polygon::new(vec![
        cloned_point,
        Point::destination_point(&point, 8.0 / 4.0 * PI, meters_to_radians(SEARCH_RADIUS)),
        Point::destination_point(&point, 7.0 / 4.0 * PI, meters_to_radians(SEARCH_RADIUS)),
        Point::destination_point(&point, 6.0 / 4.0 * PI, meters_to_radians(SEARCH_RADIUS)),
        cloned_point,
    ])
}
