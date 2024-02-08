use std::{fs::File, io::BufWriter, time::Instant};

use clap::Parser;
use osm_test::routing::{
    ch::{
        contractor::Contractor,
        graph_cleaner::{remove_edge_to_self, removing_double_edges},
    },
    graph::Graph,
    naive_graph::NaiveGraph,
};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    fmi_path: String,
    /// Path of contracted_graph (output)
    #[arg(short, long)]
    contracted_graph: String,
}

fn main() {
    let args = Args::parse();

    let naive_graph = NaiveGraph::from_fmi_file(args.fmi_path.as_str());
    // let naive_graph = NaiveGraph::from_gr_file("tests/data/fmi/USA-road-d.NY.gr");
    let mut graph = Graph::from_edges(&naive_graph.edges);
    removing_double_edges(&mut graph);
    remove_edge_to_self(&mut graph);

    let start = Instant::now();
    let contracted_graph = Contractor::get_contracted_graph(&graph);
    println!("Generating ch took {:?}", start.elapsed());

    let writer = BufWriter::new(File::create(args.contracted_graph).unwrap());
    bincode::serialize_into(writer, &contracted_graph).unwrap();
}
