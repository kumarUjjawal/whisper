use crate::ws::ws_routes;
use axum::{routing::get, Router};

pub fn get_routes() -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .merge(ws_routes())
}
