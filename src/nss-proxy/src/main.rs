use crate::config::Config;
use log::info;

mod api_types;
mod config;
mod error;
mod handler;
mod http_client;
mod logging;
mod server;
mod utils;

const VERSION: &str = env!("CARGO_PKG_VERSION");

// TODO change to /run after testing
static PROXY_SOCKET: &str = "/tmp/rauthy_proxy.sock";
// static PROXY_SOCKET: &str = "/run/rauthy/rauthy_proxy.sock";

fn main() -> anyhow::Result<()> {
    Config::load()?;

    logging::init()?;
    info!("Rauthy NSS Proxy v {VERSION}");

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

    rt.block_on(async { server::run().await })?;

    Ok(())
}
