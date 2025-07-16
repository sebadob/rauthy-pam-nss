use crate::api_types::Getent;
use crate::handler::{ApiResponse, fetch_getent};
use axum::extract::Path;
use log::info;

pub async fn get_groups() -> ApiResponse {
    info!("get all groups");
    fetch_getent(Getent::Groups).await
}

pub async fn get_group_by_gid(Path(gid): Path<u32>) -> ApiResponse {
    info!("get group id: {gid}");
    fetch_getent(Getent::GroupId(gid)).await
}

pub async fn get_group_by_name(Path(groupname): Path<String>) -> ApiResponse {
    info!("get group name: {groupname}");
    fetch_getent(Getent::Groupname(groupname)).await
}
