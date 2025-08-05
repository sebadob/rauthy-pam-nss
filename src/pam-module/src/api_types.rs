use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct PamLoginRequest {
    pub host_id: String,
    pub host_secret: String,
    pub username: String,
    pub password: Option<String>,
    pub remote_password: Option<String>,
    pub webauthn_code: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PamPreflightRequest {
    pub host_id: String,
    pub host_secret: String,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct PamPreflightResponse {
    pub login_allowed: bool,
    pub local_password_only: bool,
    pub mfa_required: bool,
}

#[derive(Debug, Serialize)]
pub struct PamMfaStartRequest {
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct PamMfaFinishRequest {
    pub user_id: String,
    pub data: WebauthnAuthFinishRequest,
}

#[derive(Debug, Deserialize)]
pub struct WebauthnAuthStartResponse {
    pub code: String,
    pub rcr: webauthn_rs::prelude::RequestChallengeResponse,
    pub user_id: String,
    // pub exp: u64,
}

#[derive(Debug, Serialize)]
pub struct WebauthnAuthFinishRequest {
    pub code: String,
    pub data: webauthn_rs::prelude::PublicKeyCredential,
}

#[derive(Debug, Deserialize)]
pub struct WebauthnServiceReq {
    pub code: String,
    // pub user_id: String,
}
