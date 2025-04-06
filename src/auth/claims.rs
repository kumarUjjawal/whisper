use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub user_id: String,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub aud: String,
    pub iss: String,
    pub exp: usize,
    pub iat: usize,
}
