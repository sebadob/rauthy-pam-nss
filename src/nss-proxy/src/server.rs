use crate::config::Config;
use crate::handler;
use axum::{Router, routing::get};
use std::path::PathBuf;
use tokio::net::UnixListener;

pub async fn run(config: Config) -> anyhow::Result<()> {
    let path = PathBuf::from(&config.listen_addr);

    let _ = tokio::fs::remove_file(&path).await;
    tokio::fs::create_dir_all(path.parent().unwrap()).await?;

    let uds = UnixListener::bind(path.clone())?;
    let app = Router::new()
        .route("/", get(handler::handler))
        .route("/getent", get(handler::get_getent))
        .into_make_service();

    axum::serve(uds, app).await?;

    Ok(())
}
