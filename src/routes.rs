use axum::{routing::get, Router};

pub fn get_routes() -> Router {
    Router::new().route("/health", get(|| async { "OK" }))
}
