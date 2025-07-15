use serde::{Deserialize, Serialize};
use std::net::IpAddr;

// #[derive(Debug, Deserialize)]
// struct ErrorResponse {
//     timestamp: i64,
//     error: String,
//     message: String,
// }

#[derive(Debug, Serialize)]
pub struct PamLoginRequest {
    pub host_id: String,
    pub host_secret: String,
    pub username: String,
    pub password: Option<String>,
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

#[derive(Debug, Serialize)]
pub enum Getent {
    Users,
    Groups,
    Username(String),
    UserId(u32),
    Groupname(String),
    GroupId(u32),
    Hosts,
    Hostname(String),
    HostIp(IpAddr),
}

#[derive(Debug, Serialize)]
pub struct PamGetentRequest {
    pub host_id: String,
    pub host_secret: String,
    pub getent: Getent,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct PamHostResponse {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub addresses: Vec<IpAddr>,
}

#[derive(Debug, Deserialize)]
pub struct PamGroupResponse {
    pub id: u32,
    pub name: String,
    // Vec<{username}>
    pub members: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PamUserResponse {
    pub id: u32,
    pub name: String,
    pub gid: u32,
    pub email: String,
    pub shell: String,
}

#[derive(Debug, Deserialize)]
pub enum PamGetentResponse {
    Users(Vec<PamUserResponse>),
    User(PamUserResponse),
    Groups(Vec<PamGroupResponse>),
    Group(PamGroupResponse),
    Hosts(Vec<PamHostResponse>),
    Host(PamHostResponse),
}
