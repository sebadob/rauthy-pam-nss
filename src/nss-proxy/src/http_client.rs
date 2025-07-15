use crate::VERSION;
use crate::api_types::{Getent, GetentRequest, GetentResponse};
use crate::config::Config;
use crate::error::{Error, ErrorType};
use crate::handler::ApiResponse;
use crate::utils::serialize;
use axum::body::Body;
use axum::response::Response;
use log::info;
use reqwest::tls::Version;
use std::sync::OnceLock;
use std::time::Duration;

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

pub struct HttpClient;

impl HttpClient {
    pub fn init() {
        let builder = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(10))
            // .http2_prior_knowledge()
            .min_tls_version(Version::TLS_1_2)
            .user_agent(format!("Rauthy NSS Proxy v{VERSION}"));

        let client = if !Config::get().danger_allow_unsecure {
            builder
                .https_only(true)
                .danger_accept_invalid_certs(false)
                .danger_accept_invalid_hostnames(false)
        } else {
            builder
        }
            .build()
            .unwrap();

        CLIENT.set(client).unwrap();
    }

    pub async fn getent(getent: Getent) -> ApiResponse {
        // TODO impl and check local cache first

        let config = Config::get();
        let url = format!("{}auth/v1/pam/getent", config.rauthy_url);

        let payload = GetentRequest {
            host_id: &config.host_id,
            host_secret: &config.host_secret,
            getent,
        };

        let res = CLIENT
            .get()
            .unwrap()
            .post(url)
            .json(&payload)
            .send()
            .await?;
        if res.status().is_success() {
            let resp = res.json::<GetentResponse>().await?;
            info!("{resp:?}");
            let bytes = serialize(&resp)?;
            Ok(Response::builder()
                .status(200)
                .body(Body::from(bytes))
                .unwrap())
        } else {
            let msg = res.text().await?;
            Err(Error::new(ErrorType::Connection, msg))
        }
    }
}
