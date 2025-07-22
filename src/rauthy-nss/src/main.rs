use crate::cache::Cache;
use crate::config::{CONFIG_PATH, Config};
use log::info;
use std::sync::atomic::AtomicBool;

mod api_types;
mod cache;
mod config;
mod error;
mod groups_local;
mod handler;
mod health_check;
mod http_client;
mod logging;
mod server;
mod utils;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(debug_assertions)]
static PROXY_SOCKET: &str = "/tmp/rauthy/rauthy_proxy.sock";
#[cfg(not(debug_assertions))]
static PROXY_SOCKET: &str = "/run/rauthy/rauthy_proxy.sock";

pub static RAUTHY_HEALTHY: AtomicBool = AtomicBool::new(false);

fn main() -> anyhow::Result<()> {
    Config::load()?;

    logging::init()?;
    info!("Rauthy NSS Proxy v{VERSION}");
    info!("Using config file from {CONFIG_PATH}");

    Cache::init();
    http_client::HttpClient::init();

    let workers = Config::get().workers;
    let rt = if workers == 1 {
        tokio::runtime::Builder::new_current_thread()
    } else {
        tokio::runtime::Builder::new_multi_thread()
    }
    .enable_all()
    .worker_threads(workers)
    .build()?;

    rt.block_on(async {
        Cache::spawn_ticker();

        health_check::wait_until_healthy().await;
        health_check::spawn_health_checker();

        server::run().await
    })?;

    info!("Shutting down");

    Ok(())
}
