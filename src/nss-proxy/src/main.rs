use crate::config::Config;
use log::info;
use std::path::PathBuf;

mod config;
mod handler;
mod logging;
mod server;
mod test;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load().await?;
    logging::init(&config)?;

    info!("Hello World!");

    let path = PathBuf::from(&config.listen_addr);
    test::spawn_tests(path);

    server::run(config).await?;

    Ok(())
}
