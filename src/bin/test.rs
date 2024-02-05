use clap::Parser;
use indicatif::ProgressIterator;
use osm_test::routing::{
    fast_graph::FastGraph,
    graph::Graph,
    naive_graph::NaiveGraph,
    route::{RouteResponse, RouteValidationRequest, Routing},
    simple_algorithms::dijkstra::Dijkstra,
};
use serde_derive::{Deserialize, Serialize};
use std::{
    fs::File,
    io::BufReader,
    time::{Duration, Instant},
    usize,
};

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

#[derive(Clone, Serialize, Deserialize)]
pub struct ContractedGraph {
    pub graph: Graph,
    pub map: Vec<((u32, u32), Vec<(u32, u32)>)>,
}

fn main() {
    let args = Args::parse();

    let graph = NaiveGraph::from_file(args.fmi_path.as_str());
    // let _graph = Graph::from_naive_graph(&naive_graph);

    // let reader = BufReader::new(File::open("graph.json").unwrap());
    // let contraced_graph: ContractedGraph = serde_json::from_reader(reader).unwrap();
    // let contraced_graph = contraced_graph.graph;
    // let edges: Vec<_> = contraced_graph
    //     .forward_edges
    //     .into_iter()
    //     .flatten()
    //     .collect();
    // let nodes = naive_graph.nodes.clone();
    // let contracted_graph = NaiveGraph { nodes, edges };
    // let contracted_graph = FastGraph::new(&contracted_graph);

    let graph = Graph::from_edges(&graph.edges);
    let graph = FastGraph::from_graph(&graph);

    let algorithms: Vec<(&str, Box<dyn Routing>)> = vec![
        // (
        //     "bidirectional a star with with zero",
        //     Box::new(BiAStarWithZero::new(&graph)),
        // ),
        // ("ch", Box::new(BiAStarWithZero::new(&contracted_graph))),
        // (
        //     "a star with landmarks",
        //     Box::new(AStarWithLandmarks::new(&graph)),
        // ),
        ("dijkstra", Box::new(Dijkstra::new(&graph))),
        // ("a star with zero", Box::new(AStarWithZero::new(&graph))),
        // (
        //     "a star with distance",
        //     Box::new(ASTarWithDistance::new(&graph)),
        // ),
        // (
        //     "bidirectional a star with landmarks",
        //     Box::new(BiAStarWithLandmarks::new(&graph)),
        // ),
    ];

    let mut algorithms: Vec<_> = algorithms
        .iter()
        .map(|(name, algorithm)| (name, algorithm, Vec::new(), Vec::new(), Vec::new()))
        .collect();

    let reader = BufReader::new(File::open(args.tests_path.as_str()).unwrap());
    let tests: Vec<RouteValidationRequest> = serde_json::from_reader(reader).unwrap();

    tests
        .iter()
        .take(args.number_of_tests as usize)
        .progress_count(args.number_of_tests as u64)
        .for_each(|validation_request| {
            let request = &validation_request.request;

            for (_, routing_algorithm, times, scanned, legal) in algorithms.iter_mut() {
                let before = Instant::now();
                let response = routing_algorithm.get_route(request);
                times.push(before.elapsed());

                scanned.push(
                    response
                        .data
                        .iter()
                        .map(|edpanded_ids| edpanded_ids.get_scanned_points().len())
                        .sum::<usize>(),
                );

                let mut this_legal = 0.0;
                if !response_is_legal(validation_request, response, &graph) {
                    this_legal = 1.0;
                }

                legal.push(this_legal);
            }
        });

    for (name, _, times, scanned, legal) in algorithms.iter() {
        println!(
            "{:<40} legal? {:>2.2?}% {:>4.3}ms, avgscan: {:>9}",
            name,
            legal.iter().sum::<f32>() / (legal.len() as f32) * 100.0,
            (times.iter().sum::<Duration>() / times.len() as u32).as_secs_f64() * 1_000.0,
            scanned.iter().sum::<usize>() / scanned.len()
        );
    }
}

fn response_is_legal(
    request: &RouteValidationRequest,
    response: RouteResponse,
    _graph: &FastGraph,
) -> bool {
    let mut response_cost = None;
    if let Some(route) = response.route {
        response_cost = Some(route.cost);
    }

    if request.cost != response_cost {
        println!("illegal route cost {:?} {:?}", request.cost, response_cost);
    }

    request.cost == response_cost
}
