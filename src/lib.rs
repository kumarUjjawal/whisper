pub mod auth;
pub mod db;
pub mod entity;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod ws;

use axum::http::Method;
use axum::{routing::get, Router};
use dotenvy::dotenv;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber;

pub async fn run() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let db = db::connect_database().await;

    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let firebase_api_key = std::env::var("FIREBASE_API_KEY").expect("FIREBASE_API_KEY must be set");
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(|| async { "Whisper Chat" }))
        .merge(routes::get_routes(db.clone()))
        .merge(auth::routes::configure_auth_routes(
            db,
            &firebase_api_key,
            jwt_secret,
        ))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("🚀 Server running on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
