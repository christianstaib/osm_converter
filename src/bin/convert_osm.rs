use clap::Parser;
use osm_converter::sphere::geometry::planet::Planet;

/// Parse parameters for OSM converter
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of the .pbf input file
    #[arg(short, long)]
    input: String,
    /// Path of the geojson output file
    #[arg(short, long)]
    output: String,
}

fn main() {
    let args = Args::parse();

    let planet = Planet::from_osm_file(args.input.as_str());
    planet.to_geojson_file(args.output.as_str())
}
