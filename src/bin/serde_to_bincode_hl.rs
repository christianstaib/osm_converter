use std::{
    fs::File,
    io::{BufReader, BufWriter},
    time::{Duration, Instant},
};

use clap::Parser;
use indicatif::ProgressIterator;
use osm_test::routing::{hl::hub_graph::HubGraph, route::RouteValidationRequest};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    hub_graph: String,
    /// Path of .fmi file
    #[arg(short, long)]
    test_path: String,
}

fn main() {
    let args = Args::parse();

    let reader = BufReader::new(File::open(args.test_path.as_str()).unwrap());
    let tests: Vec<RouteValidationRequest> = serde_json::from_reader(reader).unwrap();

    let reader = BufReader::new(File::open(args.hub_graph).unwrap());
    let hub_graph: HubGraph = serde_json::from_reader(reader).unwrap();

    let writer = BufWriter::new(File::create(args.hub_graph).unwrap());
    bincode::serialize_into(writer, &hub_graph);

    println!("avg label size is {}", hub_graph.get_avg_label_size());

    let mut time_hl = Vec::new();
    tests.iter().progress().for_each(|test| {
        let start = Instant::now();
        let cost = hub_graph.get_cost(&test.request);
        time_hl.push(start.elapsed());

        assert_eq!(cost, test.cost);
    });

    println!("all correct");

    println!(
        "took {:?} per search",
        time_hl.iter().sum::<Duration>() / time_hl.len() as u32
    );
}
