pub mod db;
pub mod entity;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod ws;

use axum::{routing::get, Router};
use dotenvy::dotenv;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber;

pub async fn run() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let db = db::connect_database().await;

    let app = Router::new()
        .route("/", get(|| async { "Whisper Chat" }))
        .merge(routes::get_routes(db));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("🚀 Server running on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
