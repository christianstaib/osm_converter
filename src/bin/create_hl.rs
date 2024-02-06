use std::{
    fs::File,
    io::{BufReader, BufWriter},
    time::Instant,
};

use clap::Parser;

use osm_test::routing::{
    ch::contractor::ContractedGraph, simple_algorithms::ch_bi_dijkstra::ChDijkstra,
};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    contracted_graph: String,
    /// Path of .fmi file
    #[arg(short, long)]
    hub_graph: String,
    /// Path of .fmi file
    #[arg(short, long)]
    hop_limit: u32,
}

fn main() {
    let args = Args::parse();

    let reader = BufReader::new(File::open(args.contracted_graph).unwrap());
    let contracted_graph: ContractedGraph = bincode::deserialize_from(reader).unwrap();

    let dijkstra = ChDijkstra::new(&contracted_graph);

    let start = Instant::now();
    let mut hub_graph = dijkstra.get_hl();
    println!("Generating hl took {:?}", start.elapsed());

    let start = Instant::now();
    hub_graph.set_predecessor();
    println!("set predecessor took {:?}", start.elapsed());

    let writer = BufWriter::new(File::create(args.hub_graph).unwrap());
    bincode::serialize_into(writer, &hub_graph).unwrap();
}
