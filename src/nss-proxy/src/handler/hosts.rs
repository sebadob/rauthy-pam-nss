use crate::api_types::Getent;
use crate::handler::ApiResponse;
use crate::http_client::HttpClient;
use axum::extract::Path;
use log::info;
use std::net::IpAddr;

pub async fn get_hosts() -> ApiResponse {
    info!("get_hosts()");
    HttpClient::getent(Getent::Hosts).await
}

pub async fn get_host_by_ip(Path(ip): Path<IpAddr>) -> ApiResponse {
    info!("get_host_by_ip() {ip}");
    HttpClient::getent(Getent::HostIp(ip)).await
}

pub async fn get_host_by_name(Path(hostname): Path<String>) -> ApiResponse {
    info!("get_host_by_name() {hostname}");
    HttpClient::getent(Getent::Hostname(hostname)).await
}
