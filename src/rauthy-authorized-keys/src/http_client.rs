use crate::config::Config;
use reqwest::tls::Version;
use std::time::Duration;

#[inline]
pub fn build(config: &Config) -> reqwest::blocking::Client {
    let builder = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(3))
        .min_tls_version(Version::TLS_1_2)
        .hickory_dns(true)
        .use_rustls_tls()
        .user_agent("Rauthy AuthorizedKeys Client");

    if config.danger_allow_insecure {
        builder
    } else {
        builder
            .https_only(true)
            .danger_accept_invalid_certs(false)
            .danger_accept_invalid_hostnames(false)
    }
    .build()
    .unwrap()
}
