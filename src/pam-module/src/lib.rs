use pam::RauthyPam;
use pamsm::{PamServiceModule, pam_module};
use std::sync::{LazyLock, OnceLock};
use std::time::Duration;

mod api_types;
mod config;
mod constants;
mod copy_dir;
mod pam;

const VERSION: &str = env!("CARGO_PKG_VERSION");

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

static RT: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_current_thread()
        .worker_threads(1)
        .max_blocking_threads(1)
        .enable_all()
        .build()
        .expect("Cannot build tokio runtime")
});

fn http_client(danger_insecure: bool) -> &'static reqwest::Client {
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(10))
            .https_only(!danger_insecure)
            .user_agent(format!("Rauthy PAM Client v{VERSION}"))
            .build()
            .unwrap()
    })
}

pam_module!(RauthyPam);
