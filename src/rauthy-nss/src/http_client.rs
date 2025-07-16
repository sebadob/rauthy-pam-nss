use crate::VERSION;
use crate::api_types::{Getent, GetentRequest, GetentResponse};
use crate::config::Config;
use crate::error::Error;
use crate::utils::serialize;
use log::{debug, error};
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
            .min_tls_version(Version::TLS_1_2)
            .hickory_dns(true)
            .use_rustls_tls()
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

    #[inline]
    pub fn client() -> &'static reqwest::Client {
        CLIENT.get().unwrap()
    }

    #[inline]
    pub async fn getent(getent: &Getent) -> Result<Option<Vec<u8>>, Error> {
        // TODO impl and check local cache first

        let config = Config::get();
        let url = format!("{}auth/v1/pam/getent", config.rauthy_url);

        let payload = GetentRequest {
            host_id: &config.host_id,
            host_secret: &config.host_secret,
            getent,
        };

        let res = match CLIENT.get().unwrap().post(url).json(&payload).send().await {
            Ok(r) => r,
            Err(err) => {
                error!("Error sending request to Rauthy: {err:?}");
                return Err(Error::from(err));
            }
        };

        if res.status().is_success() {
            let resp = res.json::<GetentResponse>().await?;
            debug!("{resp:?}");
            let bytes = serialize(&resp)?;
            Ok(Some(bytes))
        } else {
            // let msg = res.text().await?;
            // error!("{msg}");
            Ok(None)
        }
    }
}
