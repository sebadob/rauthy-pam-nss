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
// mod test;

const VERSION: &str = env!("CARGO_PKG_VERSION");

// TODO change to /run after testing
static PROXY_SOCKET: &str = "/tmp/rauthy_proxy.sock";
// static PROXY_SOCKET: &str = "/run/rauthy/rauthy_proxy.sock";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Config::load().await?;
    logging::init()?;
    http_client::HttpClient::init();

    info!("Hello World!");

    // test::spawn_tests();

    server::run().await?;

    Ok(())
}
