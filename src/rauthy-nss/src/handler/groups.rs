use crate::api_types::Getent;
use crate::handler::{ApiResponse, fetch_getent};
use axum::extract::Path;
use log::info;
use tokio::time::Instant;

pub async fn get_groups() -> ApiResponse {
    let start = Instant::now();
    match fetch_getent(Getent::Groups).await {
        Ok(res) => {
            info!(
                "get all groups - SUCCESS {} µs",
                start.elapsed().as_micros()
            );
            Ok(res)
        }
        Err(err) => {
            info!("get all groups - FAIL {} µs", start.elapsed().as_micros());
            Err(err)
        }
    }
}

pub async fn get_group_by_gid(Path(gid): Path<u32>) -> ApiResponse {
    let start = Instant::now();
    match fetch_getent(Getent::GroupId(gid)).await {
        Ok(res) => {
            info!(
                "group id {gid} - SUCCESS {} µs",
                start.elapsed().as_micros()
            );
            Ok(res)
        }
        Err(err) => {
            info!("group id {gid} - FAIL {} µs", start.elapsed().as_micros());
            Err(err)
        }
    }
}

pub async fn get_group_by_name(Path(groupname): Path<String>) -> ApiResponse {
    let start = Instant::now();
    match fetch_getent(Getent::Groupname(groupname.clone())).await {
        Ok(res) => {
            info!(
                "groupname {groupname} - SUCCESS {} µs",
                start.elapsed().as_micros()
            );
            Ok(res)
        }
        Err(err) => {
            info!(
                "groupname {groupname} - FAIL {} µs",
                start.elapsed().as_micros()
            );
            Err(err)
        }
    }
}
