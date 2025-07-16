use pam::RauthyPam;
use pamsm::{PamServiceModule, pam_module};
use std::sync::LazyLock;
use std::time::Duration;

mod api_types;
mod config;
mod pam;

const VERSION: &str = env!("CARGO_PKG_VERSION");

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10))
        .user_agent(format!("Rauthy PAM Client v{VERSION}"))
        .build()
        .unwrap()
});

static RT: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_current_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .expect("Cannot build tokio runtime")
});

pam_module!(RauthyPam);
