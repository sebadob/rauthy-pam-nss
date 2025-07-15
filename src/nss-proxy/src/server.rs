use crate::PROXY_SOCKET;
use crate::handler::get_root;
use crate::handler::groups::*;
use crate::handler::hosts::*;
use crate::handler::users::*;
use axum::{Router, routing::get};
use std::path::PathBuf;
use tokio::net::UnixListener;

pub async fn run() -> anyhow::Result<()> {
    let path = PathBuf::from(PROXY_SOCKET);

    let _ = tokio::fs::remove_file(&path).await;
    tokio::fs::create_dir_all(path.parent().unwrap()).await?;

    let uds = UnixListener::bind(path.clone())?;
    let app = Router::new()
        .route("/", get(get_root))
        .nest(
            "/getent",
            Router::new()
                .route("/groups", get(get_groups))
                .route("/groups/gid/{gid}", get(get_group_by_gid))
                .route("/groups/name/{name}", get(get_group_by_name))
                .route("/hosts", get(get_hosts))
                .route("/hosts/ip/{ip}", get(get_host_by_ip))
                .route("/hosts/name/{name}", get(get_host_by_name))
                .route("/users", get(get_users))
                .route("/users/uid/{uid}", get(get_user_by_uid))
                .route("/users/name/{name}", get(get_user_by_name)),
        )
        .into_make_service();

    axum::serve(uds, app).await?;

    Ok(())
}
