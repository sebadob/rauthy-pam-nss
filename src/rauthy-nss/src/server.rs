use crate::PROXY_SOCKET;
use crate::handler::get_root;
use crate::handler::groups::*;
use crate::handler::hosts::*;
use crate::handler::users::*;
use axum::{Router, routing::get};
use log::{debug, info};
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use tokio::fs;
use tokio::net::UnixListener;

pub async fn run() -> anyhow::Result<()> {
    let path = PathBuf::from(PROXY_SOCKET);
    let parent = path.parent().unwrap();

    let _ = tokio::fs::remove_file(&path).await;
    tokio::fs::create_dir_all(parent).await?;
    // Very important to NOT allow `w` on the dir for world to prevent the creation of new files
    // inside and the deletion of the socket itself, after it has been created. This protects the
    // socket from being spoofed, even when it's world `rw`. If the socket was spoofable, an
    // attacker could spoof responses in a way to abuse it for priv esc.
    // SELinux will only put an additional layer of protection on top, but it's not available
    // everywhere.
    fs::set_permissions(parent, Permissions::from_mode(0o755)).await?;

    let uds = UnixListener::bind(path)?;
    // The socket must be available for world.
    // It does not leak any information a normal user on the system would not be able to see anyway.
    // It only exports NSS information, that e.g. anyone could read by from /etc/passwd by default.
    //
    // TODO find a way to explicitly set the sticky bit from safe rust. Is this even possible?
    fs::set_permissions(PROXY_SOCKET, Permissions::from_mode(0o766)).await?;

    // TODO not sure why this is happening, but something changes SELinux labels for the socket
    //  between restarts. This service should run as `root`, so it will be able to restore its own
    //  label at this point, which we will do, until the main issue was found.
    restore_selinux_labels().await;

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

async fn restore_selinux_labels() {
    if fs::try_exists("/usr/sbin/restorecon").await.ok() != Some(true) {
        debug!("/usr/sbin/restorecon not found - skipping SELinux label restore");
        return;
    }

    let (parent, _) = PROXY_SOCKET.rsplit_once("/").unwrap();
    let _ = Command::new("/usr/sbin/restorecon")
        .arg("-rF")
        .arg(parent)
        .spawn();
}
