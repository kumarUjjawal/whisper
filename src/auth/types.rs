use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SendOtpRequest {
    pub phone_number: String,
}

#[derive(Deserialize)]
pub struct VerifyOtpRequest {
    pub session_info: String,
    pub code: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct FirebaseResponse {
    #[serde(rename = "idToken")]
    pub id_token: Option<String>,

    #[serde(rename = "sessionInfo")]
    pub session_info: Option<String>,

    pub error: Option<FirebaseError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FirebaseError {
    pub message: String,
}
