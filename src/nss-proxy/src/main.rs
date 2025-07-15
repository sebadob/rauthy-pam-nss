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

// TODO add clap and accept the location or socket via arg? how to inform NSS then ?

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Config::load().await?;

    logging::init()?;
    info!("Rauthy NSS Proxy v {VERSION}");

    http_client::HttpClient::init();

    server::run().await?;

    Ok(())
}
