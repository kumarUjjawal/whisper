use crate::auth::claims::Claims;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap, StatusCode},
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use reqwest;
use std::collections::{HashMap, HashSet};

pub struct FirebaseAuth(pub Claims);

#[async_trait]
impl<S> FromRequestParts<S> for FirebaseAuth
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let headers = &parts.headers;
        let token = extract_token(headers).ok_or((
            StatusCode::UNAUTHORIZED,
            "Missing or invalid Authorization header".to_string(),
        ))?;

        let claims = verify_firebase_token(&token).await.map_err(|e| {
            (
                StatusCode::UNAUTHORIZED,
                format!("Token verification failed: {}", e),
            )
        })?;

        Ok(FirebaseAuth(claims))
    }
}

fn extract_token(headers: &HeaderMap) -> Option<String> {
    let auth_header = headers.get("Authorization")?.to_str().ok()?;
    if auth_header.starts_with("Bearer ") {
        Some(auth_header.trim_start_matches("Bearer ").to_string())
    } else {
        None
    }
}

pub async fn verify_firebase_token(id_token: &str) -> Result<Claims, String> {
    let header = decode_header(id_token).map_err(|e| e.to_string())?;
    let kid = header.kid.ok_or("Missing `kid` in token header")?;

    let keys_url =
        "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com";
    let keys: HashMap<String, String> = reqwest::get(keys_url)
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let public_key_pem = keys.get(&kid).ok_or("Public key not found")?;

    let decoding_key =
        DecodingKey::from_rsa_pem(public_key_pem.as_bytes()).map_err(|e| e.to_string())?;

    let project_id = std::env::var("FIREBASE_PROJECT_ID")
        .map_err(|_| "Missing FIREBASE_PROJECT_ID env var".to_string())?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[&project_id]);
    let mut issuers = HashSet::new();
    issuers.insert(format!("https://securetoken.google.com/{}", project_id));
    validation.iss = Some(issuers);

    let token_data =
        decode::<Claims>(id_token, &decoding_key, &validation).map_err(|e| e.to_string())?;

    Ok(token_data.claims)
}
