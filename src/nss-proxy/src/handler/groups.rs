use crate::api_types::Getent;
use crate::handler::ApiResponse;
use crate::http_client::HttpClient;
use axum::extract::Path;
use log::info;

pub async fn get_groups() -> ApiResponse {
    info!("get_groups()");
    HttpClient::getent(Getent::Groups).await
}

pub async fn get_group_by_gid(Path(gid): Path<u32>) -> ApiResponse {
    info!("get_group_by_gid() {gid}");
    HttpClient::getent(Getent::GroupId(gid)).await
}

pub async fn get_group_by_name(Path(groupname): Path<String>) -> ApiResponse {
    info!("get_group_by_name()");
    HttpClient::getent(Getent::Groupname(groupname)).await
}
