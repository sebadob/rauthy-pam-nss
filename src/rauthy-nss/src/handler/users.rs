use crate::api_types::Getent;
use crate::handler::{ApiResponse, fetch_getent};
use axum::extract::Path;
use log::info;

pub async fn get_users() -> ApiResponse {
    info!("get all users");
    fetch_getent(Getent::Users).await
}

pub async fn get_user_by_uid(Path(uid): Path<u32>) -> ApiResponse {
    info!("get user id: {uid}");
    fetch_getent(Getent::UserId(uid)).await
}

pub async fn get_user_by_name(Path(username): Path<String>) -> ApiResponse {
    info!("get user name: {username}");
    fetch_getent(Getent::Username(username)).await
}
