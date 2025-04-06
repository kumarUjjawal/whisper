use crate::auth::firebase_auth::FirebaseAuth;
use crate::auth::service;
use crate::auth::types::{SendOtpRequest, VerifyOtpRequest};
use crate::entity::users;
use axum::extract::State;
use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json as JsonResponse,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
pub async fn send_otp_handler(Json(payload): Json<SendOtpRequest>) -> Response {
    let response = service::send_otp(payload.phone_number).await;

    if response.error.is_some() {
        // Return error with HTTP 422
        return (StatusCode::UNPROCESSABLE_ENTITY, JsonResponse(response)).into_response();
    }

    // Return success with HTTP 200
    (StatusCode::OK, JsonResponse(response)).into_response()
}

pub async fn verify_otp_handler(Json(payload): Json<VerifyOtpRequest>) -> Response {
    let response = service::verify_otp(payload.session_info, payload.code).await;

    if response.error.is_some() {
        return (StatusCode::UNPROCESSABLE_ENTITY, JsonResponse(response)).into_response();
    }

    (StatusCode::OK, JsonResponse(response)).into_response()
}

#[derive(serde::Serialize)]
pub struct UserResponse {
    id: i32,
    username: String,
    phone_number: String,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn verify_token_and_upsert_user(
    State(db): State<DatabaseConnection>,
    FirebaseAuth(claims): FirebaseAuth,
) -> Result<JsonResponse<UserResponse>, (StatusCode, String)> {
    let phone_number = claims.phone_number.clone().ok_or((
        StatusCode::BAD_REQUEST,
        "No phone number in token".to_string(),
    ))?;

    let existing_user = users::Entity::find()
        .filter(users::Column::PhoneNumber.eq(phone_number.clone()))
        .one(&db)
        .await
        .map_err(internal_error)?;

    let user = if let Some(user) = existing_user {
        user
    } else {
        let now = Utc::now();
        let new_user = users::ActiveModel {
            username: Set(generate_username(&phone_number)),
            phone_number: Set(phone_number.clone()),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        };

        new_user.insert(&db).await.map_err(internal_error)?
    };

    Ok(JsonResponse(UserResponse {
        id: user.id,
        username: user.username,
        phone_number: user.phone_number,
        created_at: user.created_at,
        updated_at: user.updated_at,
    }))
}

fn generate_username(phone: &str) -> String {
    format!("user_{}", &phone[phone.len().saturating_sub(4)..])
}

fn internal_error<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
