use std::{fs::File, io::BufWriter, io::Write};

use clap::Parser;
use indicatif::ProgressIterator;
use osm_test::routing::{
    fast_graph::FastGraph,
    graph::Graph,
    naive_graph::NaiveGraph,
    route::{RouteRequest, RouteValidationRequest, Routing},
    simple_algorithms::dijkstra::Dijkstra,
};
use rand::Rng;
use rayon::iter::{ParallelBridge, ParallelIterator};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    fmi_path: String,
    /// Path of .fmi file
    #[arg(short, long)]
    tests_path: String,
    /// Number of tests to be run
    #[arg(short, long)]
    number_of_tests: u32,
}

fn main() {
    let args = Args::parse();

    let graph = NaiveGraph::from_file(args.fmi_path.as_str());
    let graph = Graph::from_naive_graph(&graph);
    let graph = FastGraph::from_graph(&graph);
    let dijkstra = Dijkstra::new(&graph);

    let routes: Vec<_> = (0..args.number_of_tests)
        .progress()
        .par_bridge()
        .map(|_| {
            let mut rng = rand::thread_rng();
            let request = RouteRequest {
                source: rng.gen_range(0..graph.num_nodes) as u32,
                target: rng.gen_range(0..graph.num_nodes) as u32,
            };

            let response = dijkstra.get_route(&request);
            let mut cost = None;
            if let Some(route) = response.route {
                cost = Some(route.cost);
            }

            RouteValidationRequest { request, cost }
        })
        .collect();

    let mut writer = BufWriter::new(File::create(args.tests_path.as_str()).unwrap());
    serde_json::to_writer(&mut writer, &routes).unwrap();
    writer.flush().unwrap();
}
