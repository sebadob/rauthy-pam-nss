use libnss::{libnss_group_hooks, libnss_host_hooks, libnss_passwd_hooks};
use log::LevelFilter;
use std::process;
use syslog::{BasicLogger, Facility, Formatter3164};

mod api_types;
mod nss;
mod uds;

// TODO change to /run after testing
// static PROXY_SOCKET: &str = "/tmp/rauthy_proxy.sock";
static PROXY_SOCKET: &str = "/run/rauthy/rauthy_proxy.sock";

pub struct RauthyNss;

fn init_syslog() {
    let formatter = Formatter3164 {
        facility: Facility::LOG_SYSLOG,
        hostname: None,
        process: "Rauthy NSS".into(),
        pid: process::id(),
    };

    let logger = syslog::unix(formatter).expect("could not connect to syslog");
    let _ = log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
        .map(|()| log::set_max_level(LevelFilter::Info));
}

libnss_passwd_hooks!(rauthy, RauthyNss);
// libnss_shadow_hooks!(rauthy, RauthyNss);
libnss_group_hooks!(rauthy, RauthyNss);
libnss_host_hooks!(rauthy, RauthyNss);
