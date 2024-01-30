use std::{
    fs::File,
    io::{BufReader, BufWriter},
    time::Instant,
};

use clap::Parser;
use osm_test::routing::hl::hub_graph::HubGraph;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    hub_graph: String,
    /// Path of .fmi file
    #[arg(short, long)]
    pruned_hub_graph: String,
}

fn main() {
    let args = Args::parse();

    let reader = BufReader::new(File::open(args.hub_graph).unwrap());
    let mut hub_graph: HubGraph = bincode::deserialize_from(reader).unwrap();

    let start = Instant::now();
    hub_graph.prune();
    println!("took {:?} to prune graph", start.elapsed());

    let writer = BufWriter::new(File::create(args.pruned_hub_graph).unwrap());
    bincode::serialize_into(writer, &hub_graph).unwrap();
}
