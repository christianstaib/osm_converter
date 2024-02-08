use clap::Parser;
use osm_test::sphere::{geometry::planet::Planet, graph::graph_generator::generate_network};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    input: String,
    /// Path of .fmi file
    #[arg(short, long)]
    num_nodes: u32,
    /// Path of .fmi file
    #[arg(short, long)]
    output_network: String,
    /// Path of .fmi file
    #[arg(short, long)]
    output_geojson: String,
    /// Path of .png file
    #[arg(short, long)]
    output_image: String,
}

fn main() {
    let args = Args::parse();

    let planet = Planet::from_geojson_file(args.input.as_str()).unwrap();

    generate_network(
        args.num_nodes,
        &planet,
        args.output_network.as_str(),
        args.output_geojson.as_str(),
        args.output_image.as_str(),
    );
}
