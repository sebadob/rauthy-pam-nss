use crate::api_types::Getent;
use crate::handler::{ApiResponse, fetch_getent};
use axum::extract::Path;
use log::info;
use tokio::time::Instant;

pub async fn get_users() -> ApiResponse {
    let start = Instant::now();
    match fetch_getent(Getent::Users).await {
        Ok(res) => {
            info!("get all users - SUCCESS {} µs", start.elapsed().as_micros());
            Ok(res)
        }
        Err(err) => {
            info!("get all users - FAIL {} µs", start.elapsed().as_micros());
            Err(err)
        }
    }
}

pub async fn get_user_by_uid(Path(uid): Path<u32>) -> ApiResponse {
    let start = Instant::now();
    match fetch_getent(Getent::UserId(uid)).await {
        Ok(res) => {
            info!(
                "users id {uid} - SUCCESS {} µs",
                start.elapsed().as_micros()
            );
            Ok(res)
        }
        Err(err) => {
            info!("users id {uid} - FAIL {} µs", start.elapsed().as_micros());
            Err(err)
        }
    }
}

pub async fn get_user_by_name(Path(username): Path<String>) -> ApiResponse {
    let start = Instant::now();
    match fetch_getent(Getent::Username(username.clone())).await {
        Ok(res) => {
            info!(
                "username: {username} - SUCCESS {} µs",
                start.elapsed().as_micros()
            );
            Ok(res)
        }
        Err(err) => {
            info!(
                "username: {username} - FAIL {} µs",
                start.elapsed().as_micros()
            );
            Err(err)
        }
    }
}
