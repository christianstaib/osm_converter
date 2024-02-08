use osm_test::sphere::{
    geometry::planet::Planet, spatial_partition::polygon_spatial_partition::PolygonSpatialPartition,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

#[test]
fn planet_grid() {
    const PLANET_PATH: &str = "tests/data/geojson/planet.geojson";
    let planet = Planet::from_geojson_file(PLANET_PATH).unwrap();

    // generating grid
    let mut planet_grid = PolygonSpatialPartition::new(50);
    planet_grid.add_polygons(&planet.polygons);

    // updating midpoints
    println!("updating midpoints");

    // test if points known to be on land are correctly categorized
    const ON_LAND_PATH: &str = "tests/data/geojson/points_on_land.geojson";
    let on_land = Planet::from_geojson_file(ON_LAND_PATH).unwrap();
    on_land.points.par_iter().for_each(|point| {
        assert!(
            planet_grid.is_on_polygon(point),
            "point {} should be on land but isn't",
            point
        )
    });

    // test if points known to be on water are correctly categorized
    const ON_WATER_PATH: &str = "tests/data/geojson/points_on_water.geojson";
    let on_water = Planet::from_geojson_file(ON_WATER_PATH).unwrap();
    on_water.points.par_iter().for_each(|point| {
        assert!(
            !planet_grid.is_on_polygon(point),
            "point {} should be on water but isn't",
            point
        )
    });
}
