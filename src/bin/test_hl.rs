use std::{
    fs::File,
    io::BufReader,
    time::{Duration, Instant},
};

use clap::Parser;
use indicatif::ProgressIterator;
use osm_test::routing::{
    graph::Graph, hl::hub_graph::HubGraph, naive_graph::NaiveGraph, path::RouteValidationRequest,
};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    hub_graph: String,
    /// Path of .fmi file
    #[arg(short, long)]
    fmi_path: String,
    /// Path of .fmi file
    #[arg(short, long)]
    test_path: String,
}

fn main() {
    let args = Args::parse();

    let graph = NaiveGraph::from_fmi_file(args.fmi_path.as_str());
    let graph = Graph::from_edges(&graph.edges);

    let reader = BufReader::new(File::open(args.test_path.as_str()).unwrap());
    let tests: Vec<RouteValidationRequest> = serde_json::from_reader(reader).unwrap();

    let reader = BufReader::new(File::open(args.hub_graph).unwrap());
    let hub_graph: HubGraph = bincode::deserialize_from(reader).unwrap();

    println!("avg label size is {}", hub_graph.get_avg_label_size());

    let mut time_hl = Vec::new();
    tests.iter().progress().for_each(|test| {
        let start = Instant::now();
        let path = hub_graph.get_route(&test.request);
        time_hl.push(start.elapsed());

        let mut cost = None;
        if let Some(route) = path {
            cost = Some(route.weight);
            graph.validate_route(&test.request, &route);
        }
        assert_eq!(cost, test.cost);
    });

    println!("all correct");

    println!(
        "took {:?} per search",
        time_hl.iter().sum::<Duration>() / time_hl.len() as u32
    );
}
