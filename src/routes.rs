use crate::ws::ws_routes;
use axum::routing::get;
use axum::Router;
use sea_orm::DatabaseConnection;

pub fn get_routes(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .merge(ws_routes(db))
}
