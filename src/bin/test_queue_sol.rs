use std::{fs::File, io::BufRead, io::BufReader};

use clap::Parser;
use indicatif::ProgressIterator;
use osm_test::routing::{
    ch::contractor::ContractedGraph,
    fast_graph::FastGraph,
    hl::hub_graph::HubGraph,
    naive_graph::NaiveGraph,
    route::{RouteRequest, Routing},
    simple_algorithms::{bi_dijkstra::BiDijkstra, ch_bi_dijkstra::ChDijkstra, dijkstra::Dijkstra},
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
    fmi_ch_path: String,
    /// Path of .fmi file
    #[arg(short, long)]
    fmi_hl_path: String,
    /// Path of .fmi file
    #[arg(short, long)]
    queue_path: String,
    /// Path of .fmi file
    #[arg(short, long)]
    sol_path: String,
}

fn main() {
    let args = Args::parse();

    let graph = NaiveGraph::from_file(args.fmi_path.as_str());
    let graph = FastGraph::from_naive_graph(&graph);

    let dijkstra = Dijkstra::new(&graph);

    let bi_dijkstra = BiDijkstra::new(&graph);

    let reader = BufReader::new(File::open(args.fmi_ch_path).unwrap());
    let ch_graph: ContractedGraph = bincode::deserialize_from(reader).unwrap();
    let ch_bi_dijkstra = ChDijkstra::new(&ch_graph);

    let reader = BufReader::new(File::open(args.fmi_hl_path).unwrap());
    let hl_graph: HubGraph = bincode::deserialize_from(reader).unwrap();

    let queue: Vec<_> = BufReader::new(File::open(args.queue_path).unwrap())
        .lines()
        .map(|s| s.ok())
        .filter_map(|s| s)
        .map(|s| {
            s.split_whitespace()
                .map(|num| num.parse::<u32>().unwrap())
                .collect::<Vec<_>>()
        })
        .collect();

    let sol: Vec<_> = BufReader::new(File::open(args.sol_path).unwrap())
        .lines()
        .map(|s| s.ok())
        .filter_map(|s| s)
        .map(|s| s.parse::<i32>().unwrap())
        .collect();

    for (source_target, true_cost) in queue.iter().zip(sol.iter()).progress() {
        let request = RouteRequest {
            source: source_target[0],
            target: source_target[1],
        };

        // test dijkstra
        let response = dijkstra.get_route(&request);
        let mut cost: i32 = -1;
        if let Some(route) = response.route {
            cost = route.cost as i32;
        }
        assert_eq!(true_cost, &cost, "dijkstra wrong");

        // test bi dijkstra
        let response = bi_dijkstra.get_route(&request);
        let mut cost: i32 = -1;
        if let Some(route) = response.route {
            cost = route.cost as i32;
        }
        assert_eq!(true_cost, &cost, "bi dijkstra wrong");

        // test ch dijkstra
        let response = ch_bi_dijkstra.get_route(&request);
        let mut cost: i32 = -1;
        if let Some(route) = response {
            cost = route.cost as i32;
        }
        assert_eq!(true_cost, &cost, "ch dijkstra wrong");

        // test hl
        let response = hl_graph.get_cost(&request);
        let mut cost: i32 = -1;
        if let Some(this_cost) = response {
            cost = this_cost as i32;
        }
        assert_eq!(true_cost, &cost, "bi dijkstra wrong");
    }
}
