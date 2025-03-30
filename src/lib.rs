pub mod db;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod ws;

use axum::{routing::get, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber;

pub async fn run() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(|| async { "Whisper Chat" }))
        .merge(routes::get_routes());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let server = axum::Server::bind(&addr).serve(app.into_make_service());
    server.await.unwrap();

    println!("Server running on http://{}", addr);
}
