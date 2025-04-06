use axum::{routing::post, Router};
use sea_orm::DatabaseConnection;

use crate::auth::handlers::{send_otp_handler, verify_otp_handler};

pub fn configure_auth_routes(
    _db: DatabaseConnection,
    _firebase_credentials_path: &str,
    _jwt_secret: String,
) -> Router {
    Router::new()
        .route("/auth/send-otp", post(send_otp_handler))
        .route("/auth/verify-otp", post(verify_otp_handler))
}
