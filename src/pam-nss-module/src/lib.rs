use libnss::{libnss_group_hooks, libnss_host_hooks, libnss_passwd_hooks, libnss_shadow_hooks};
use log::LevelFilter;
use nss::RauthyNss;
use pam::RauthyPam;
use pamsm::{PamServiceModule, pam_module};
use std::process;
use std::sync::LazyLock;
use std::time::Duration;
use syslog::{BasicLogger, Facility, Formatter3164};

mod api_types;
mod config;
mod nss;
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

fn init_syslog() {
    let formatter = Formatter3164 {
        facility: Facility::LOG_SYSLOG,
        hostname: None,
        process: "Rauthy PAM NSS".into(),
        pid: process::id(),
    };

    let logger = syslog::unix(formatter).expect("could not connect to syslog");
    let _ = log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
        .map(|()| log::set_max_level(LevelFilter::Info));
}

pam_module!(RauthyPam);

libnss_passwd_hooks!(rauthy, RauthyNss);
libnss_shadow_hooks!(rauthy, RauthyNss);
libnss_group_hooks!(rauthy, RauthyNss);
libnss_host_hooks!(rauthy, RauthyNss);
