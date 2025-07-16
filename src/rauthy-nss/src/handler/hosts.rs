use crate::api_types::Getent;
use crate::handler::{ApiResponse, fetch_getent};
use axum::extract::Path;
use log::info;
use std::net::IpAddr;
use tokio::time::Instant;

pub async fn get_hosts() -> ApiResponse {
    let start = Instant::now();
    match fetch_getent(Getent::Hosts).await {
        Ok(res) => {
            info!("get all hosts - SUCCESS {} µs", start.elapsed().as_micros());
            Ok(res)
        }
        Err(err) => {
            info!("get all hosts - FAIL {} µs", start.elapsed().as_micros());
            Err(err)
        }
    }
}

pub async fn get_host_by_ip(Path(ip): Path<IpAddr>) -> ApiResponse {
    let start = Instant::now();
    match fetch_getent(Getent::HostIp(ip)).await {
        Ok(res) => {
            info!("host ip: {ip} - SUCCESS {} µs", start.elapsed().as_micros());
            Ok(res)
        }
        Err(err) => {
            info!("host ip: {ip} - FAIL {} µs", start.elapsed().as_micros());
            Err(err)
        }
    }
}

pub async fn get_host_by_name(Path(hostname): Path<String>) -> ApiResponse {
    let start = Instant::now();
    match fetch_getent(Getent::Hostname(hostname.clone())).await {
        Ok(res) => {
            info!(
                "hostname: {hostname} - SUCCESS {} µs",
                start.elapsed().as_micros()
            );
            Ok(res)
        }
        Err(err) => {
            info!(
                "hostname: {hostname} - FAIL {} µs",
                start.elapsed().as_micros()
            );
            Err(err)
        }
    }
}
