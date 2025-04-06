use crate::auth::types::{FirebaseError, FirebaseResponse};
use reqwest::Client;
use serde_json::json;
use std::env;

fn get_api_key() -> String {
    env::var("FIREBASE_API_KEY").expect("FIREBASE_API_KEY must be set")
}

pub async fn send_otp(phone_number: String) -> FirebaseResponse {
    let api_key = get_api_key();
    let client = Client::new();
    let url = format!(
        "https://identitytoolkit.googleapis.com/v1/accounts:sendVerificationCode?key={}",
        api_key
    );
    let recaptcha_token = if phone_number == "+919599115751" {
        "test"
    } else {
        "REPLACE_WITH_REAL_TOKEN"
    };

    let body = json!({
        "phoneNumber": phone_number,
        "recaptchaToken": recaptcha_token
    });
    // let body = json!({
    //     "phoneNumber": phone_number,
    //     "recaptchaToken": "REPLACE_WITH_REAL_TOKEN_OR_USE_TEST_NUMBERS"
    // });

    match client.post(&url).json(&body).send().await {
        Ok(resp) => resp
            .json::<FirebaseResponse>()
            .await
            .unwrap_or(FirebaseResponse {
                id_token: None,
                session_info: None,
                error: Some(FirebaseError {
                    message: "Failed to parse Firebase response".into(),
                }),
            }),
        Err(_) => FirebaseResponse {
            id_token: None,
            session_info: None,
            error: Some(FirebaseError {
                message: "Failed to send OTP".into(),
            }),
        },
    }
}

pub async fn verify_otp(session_info: String, code: String) -> FirebaseResponse {
    let api_key = get_api_key();
    let client = Client::new();
    let url = format!(
        "https://identitytoolkit.googleapis.com/v1/accounts:signInWithPhoneNumber?key={}",
        api_key
    );

    let body = json!({
        "sessionInfo": session_info,
        "code": code
    });

    match client.post(&url).json(&body).send().await {
        Ok(resp) => resp
            .json::<FirebaseResponse>()
            .await
            .unwrap_or(FirebaseResponse {
                id_token: None,
                session_info: None,
                error: Some(FirebaseError {
                    message: "Failed to parse Firebase response".into(),
                }),
            }),
        Err(_) => FirebaseResponse {
            id_token: None,
            session_info: None,
            error: Some(FirebaseError {
                message: "OTP verification failed".into(),
            }),
        },
    }
}
