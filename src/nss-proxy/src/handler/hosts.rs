use crate::api_types::Getent;
use crate::handler::{ApiResponse, fetch_getent};
use axum::extract::Path;
use log::info;
use std::net::IpAddr;

pub async fn get_hosts() -> ApiResponse {
    info!("get all hosts");
    fetch_getent(Getent::Hosts).await
}

pub async fn get_host_by_ip(Path(ip): Path<IpAddr>) -> ApiResponse {
    info!("get host ip: {ip}");
    fetch_getent(Getent::HostIp(ip)).await
}

pub async fn get_host_by_name(Path(hostname): Path<String>) -> ApiResponse {
    info!("get host name: {hostname}");
    fetch_getent(Getent::Hostname(hostname)).await
}
