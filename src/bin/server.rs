use std::sync::Arc;
use std::time::Instant;

use osm_test::routing::fast_graph::FastGraph;
use osm_test::routing::naive_graph::NaiveGraph;
use osm_test::routing::route::RouteRequest;
use osm_test::routing::route::Routing;
use osm_test::routing::simple_algorithms::a_star_with_landmarks::AStarWithLandmarks;
use osm_test::sphere::geometry;
use osm_test::sphere::geometry::linestring::Linestring;
use osm_test::sphere::geometry::planet::Planet;
use osm_test::sphere::graph::graph::Fmi;
use serde_derive::{Deserialize, Serialize};
use warp::{http::Response, Filter};

use clap::Parser;

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    fmi_path: String,
    /// The address to bind to
    #[arg(short, long, default_value_t = String::from("127.0.0.1:3030"))]
    bind: String,
}

#[derive(Deserialize, Serialize)]
struct RouteRequestLatLon {
    from: (f64, f64), // lon, lat
    to: (f64, f64),   // lon, lat
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let cors = warp::cors()
        .allow_any_origin() // For development. For production, specify allowed origins.
        .allow_headers(vec!["Content-Type"]) // Specify allowed headers
        .allow_methods(vec!["GET", "POST", "OPTIONS"]); // Specify allowed methods

    println!("Loading graph from file");
    let time = Instant::now();
    let naive_graph = NaiveGraph::from_file(args.fmi_path.as_str());
    let graph = FastGraph::from_naive_graph(&naive_graph);
    let graph = Arc::new(graph);
    let fmi = Arc::new(Fmi::from_file(args.fmi_path.as_str()));
    println!("Finished loading graph, took {:?}.", time.elapsed());

    let frontend = warp::path::end().and(warp::fs::dir("public-html"));
    let promote = warp::post()
        .and(warp::path("route"))
        .and(warp::body::json())
        .map(move |route_request_lat_lon: RouteRequestLatLon| {
            let source = fmi.nearest(route_request_lat_lon.from.0, route_request_lat_lon.from.1);
            let target = fmi.nearest(route_request_lat_lon.to.0, route_request_lat_lon.to.1);
            let request = RouteRequest { source, target };

            let dijkstra = AStarWithLandmarks::new(&graph);
            let start = Instant::now();
            let response = dijkstra.get_route(&request);

            let extendes_ids: Vec<_> = response
                .data
                .iter()
                .flat_map(|data| data.get_scanned_points())
                .collect();
            let time = start.elapsed();

            if let Some(route) = response.route {
                let ids = &route.nodes;
                let path = fmi.convert_path(ids);
                let linesstring = Linestring::new(path);

                assert!(&route.is_valid(&graph, &request));
                let mut planet = Planet::new();
                planet.linestrings.push(linesstring);
                for id in extendes_ids {
                    let p = naive_graph.nodes[id];
                    planet.arcs.push(geometry::arc::Arc::new(&p, &p));
                }

                println!(
                    "route_request: {:>7} -> {:>7}, cost: {:>9}, took: {:>3}ms",
                    source,
                    target,
                    route.cost,
                    time.as_millis()
                );
                return Response::builder().body(planet.to_geojson_str().to_string());
            }

            Response::builder().body("".into())
        })
        .with(cors);

    let routes = frontend.or(promote);
    let address = args.bind.parse::<std::net::SocketAddr>().unwrap();
    warp::serve(routes).run(address).await
}
