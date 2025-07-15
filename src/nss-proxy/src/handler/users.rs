use crate::api_types::Getent;
use crate::handler::ApiResponse;
use crate::http_client::HttpClient;
use axum::extract::Path;
use log::info;

pub async fn get_users() -> ApiResponse {
    info!("get_users()");
    HttpClient::getent(Getent::Users).await
}

pub async fn get_user_by_uid(Path(uid): Path<u32>) -> ApiResponse {
    info!("get_user_by_uid() {uid}");
    HttpClient::getent(Getent::UserId(uid)).await
}

pub async fn get_user_by_name(Path(username): Path<String>) -> ApiResponse {
    info!("get_user_by_name() {username}");
    HttpClient::getent(Getent::Username(username)).await
}
