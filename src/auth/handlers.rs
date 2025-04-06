use crate::auth::service;
use crate::auth::types::{SendOtpRequest, VerifyOtpRequest};
use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json as JsonResponse,
};

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
