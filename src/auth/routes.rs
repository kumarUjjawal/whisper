use axum::{
    routing::{get, post},
    Router,
};
use sea_orm::DatabaseConnection;

use crate::auth::handlers::{send_otp_handler, verify_otp_handler, verify_token_and_upsert_user};

pub fn configure_auth_routes(
    db: DatabaseConnection,
    _firebase_api_key: &str,
    _jwt_secret: String,
) -> Router {
    Router::new()
        .route("/auth/send-otp", post(send_otp_handler))
        .route("/auth/verify-otp", post(verify_otp_handler))
        .route("/auth/me", get(verify_token_and_upsert_user))
        .with_state(db)
}
