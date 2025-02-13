use warp::Filter;
use std::net::SocketAddr;

pub async fn start_server() {
    // Define the health endpoint
    let health_route = warp::path("health")
        .map(|| "Server is running");

    // Serve index.html at the root path ("/")
    let index = warp::path::end().and(warp::fs::file("../dist/index.html"));

    // Define static files route for other assets in the "dist" folder
    let static_files = warp::fs::dir("../dist");

    // Combine the routes, prioritizing index.html at root
    let routes = index.or(health_route).or(static_files);

    // Bind the server to localhost:21296
    let addr = SocketAddr::from(([127, 0, 0, 1], 21296));
    println!("Starting server at http://{}", addr);

    // Start the server using warp::serve
    warp::serve(routes)
        .run(addr)
        .await;
}
