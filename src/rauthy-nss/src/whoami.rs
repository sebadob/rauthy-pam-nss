use crate::api_types::{HostDetailsResponse, HostWhoamiRequest};
use crate::config::Config;
use crate::error::{Error, ErrorType};
use crate::http_client::HttpClient;
use log::info;

pub async fn whoami() -> Result<(), Error> {
    let config = Config::get();
    let url = format!(
        "{}auth/v1/pam/hosts/{}/whoami",
        config.rauthy_url, config.host_id
    );

    let res = HttpClient::client()
        .post(url)
        .json(&HostWhoamiRequest {
            host_secret: &config.host_secret,
        })
        .send()
        .await?;

    if res.status().is_success() {
        let resp = res.json::<HostDetailsResponse>().await?;
        info!(
            r#"This Host:

hostname:  {}
force MFA: {}
ips:       {:?}
aliases:   {:?}
notes:     {}
"#,
            resp.hostname,
            resp.force_mfa,
            resp.ips,
            resp.aliases,
            resp.notes.unwrap_or_default()
        );
        Ok(())
    } else {
        let text = res.text().await?;
        Err(Error::new(ErrorType::Connection, text))
    }
}
