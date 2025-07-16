use crate::PROXY_SOCKET;
use crate::handler::get_root;
use crate::handler::groups::*;
use crate::handler::hosts::*;
use crate::handler::users::*;
use axum::{Router, routing::get};
use log::info;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tokio::fs;
use tokio::net::UnixListener;

pub async fn run() -> anyhow::Result<()> {
    let path = PathBuf::from(PROXY_SOCKET);

    let _ = tokio::fs::remove_file(&path).await;
    tokio::fs::create_dir_all(path.parent().unwrap()).await?;

    let uds = UnixListener::bind(path)?;
    // The socket must be available for world.
    // It does not leak any information that a normal user on the system would not be able
    // to see anyway. It only exports NSS information, that e.g. anyone could read anyway by
    // from /etc/passwd anyway.
    fs::set_permissions(PROXY_SOCKET, Permissions::from_mode(0o766)).await?;

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

    info!("Listening on socket {PROXY_SOCKET}");
    axum::serve(uds, app).await?;

    Ok(())
}
