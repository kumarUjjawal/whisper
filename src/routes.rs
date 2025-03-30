use crate::ws::ws_routes;
use axum::{routing::get, Router};
use sqlx::PgPool;

pub fn get_routes(pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .merge(ws_routes(pool))
}
