use crate::VERSION;
use crate::error::Error;
use axum::body::Body;
use axum::http::Response;

pub mod groups;
pub mod hosts;
pub mod users;

pub type ApiResponse = Result<Response<Body>, Error>;

pub async fn get_root() -> String {
    format!("Rauthy NSS Proxy v{VERSION}")
}
