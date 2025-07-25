use crate::api_types::{
    PamLoginRequest, PamMfaFinishRequest, PamMfaStartRequest, PamPreflightRequest,
    PamPreflightResponse, WebauthnAuthFinishRequest, WebauthnAuthStartResponse, WebauthnServiceReq,
};
use crate::config::Config;
use crate::pam::token::PamToken;
use crate::pam::{PamService, RauthyPam, sys_err, sys_info};
use crate::{CLIENT, RT};
use pamsm::{Pam, PamError, PamLibExt};
use reqwest::Url;
use std::time::Duration;
use tokio::time;
use tokio::time::Instant;
use webauthn_authenticator_rs::AuthenticatorBackend;

impl RauthyPam {
    async fn preflight(
        origin: Url,
        host_id: String,
        host_secret: String,
        username: String,
    ) -> Result<PamPreflightResponse, String> {
        let url = format!("{origin}auth/v1/pam/preflight");
        let res = CLIENT
            .post(url)
            .json(&PamPreflightRequest {
                host_id,
                host_secret,
                username: username.clone(),
            })
            .send()
            .await
            .map_err(|err| {
                let status = err.status();
                format!("HTTP {status:?} during preflight request for user '{username}': {err}",)
            })?;

        if res.status().is_success() {
            match res.json::<PamPreflightResponse>().await {
                Ok(r) => Ok(r),
                Err(err) => Err(format!("Error extracting preflight response: {err}")),
            }
        } else {
            Err(res.text().await.unwrap_or_default())
        }
    }

    async fn mfa(origin: Url, username: String) -> Result<String, String> {
        println!("Provide your Passkey");

        // This short sleep will fight a race condition.
        // The println may not appear if we try to find keys too quickly, when no key is inserted.
        time::sleep(Duration::from_millis(100)).await;

        // let ui = Cli {};
        // let mut p = Box::new(Self::webauthn_provider(&ui).await?);

        // TODO for some reason, the generic Ctap2 authenticator does not work
        //  the Mozilla is limited to USB only, which is realistically the only one that should be
        //  used anyway, but exploring if we could support cabLE as well would be nice in the future.
        let mut p = Box::<webauthn_authenticator_rs::mozilla::MozillaAuthenticator>::default();

        let url_start = format!("{origin}auth/v1/pam/mfa/start");
        let url_finish = format!("{origin}auth/v1/pam/mfa/finish");

        let res = CLIENT
            .post(url_start)
            .json(&PamMfaStartRequest { username })
            .send()
            .await
            .map_err(|err| err.to_string())?;

        let resp = if res.status().is_success() {
            res.json::<WebauthnAuthStartResponse>()
                .await
                .map_err(|err| err.to_string())?
        } else {
            let err = res.text().await.unwrap_or_default();
            eprintln!("{err}");
            return Err(err);
        };

        let timeout = resp.rcr.public_key.timeout.unwrap_or(60_000);
        let start = Instant::now();

        for _ in 0..3 {
            match p.perform_auth(
                origin.clone(),
                resp.rcr.public_key.clone(),
                start.elapsed().as_micros() as u32 - timeout,
            ) {
                Ok(pk_cred) => {
                    let res = CLIENT
                        .post(url_finish)
                        .json(&PamMfaFinishRequest {
                            user_id: resp.user_id,
                            data: WebauthnAuthFinishRequest {
                                code: resp.code,
                                data: pk_cred,
                            },
                        })
                        .send()
                        .await
                        .map_err(|err| err.to_string())?;

                    let data = if res.status().is_success() {
                        res.json::<WebauthnServiceReq>()
                            .await
                            .map_err(|err| err.to_string())?
                    } else {
                        return Err(res.text().await.unwrap_or_default());
                    };

                    return Ok(data.code);
                }
                Err(err) => {
                    eprintln!("{err:?}");
                }
            }
        }

        Err("Passkey validation failed".to_string())
    }

    async fn send_login(origin: Url, payload: PamLoginRequest) -> Result<PamToken, String> {
        let url = format!("{origin}auth/v1/pam/login");
        let res = CLIENT
            .post(url)
            .json(&payload)
            .send()
            .await
            .map_err(|err| err.to_string())?;

        if res.status().is_success() {
            res.json::<PamToken>().await.map_err(|err| err.to_string())
        } else {
            Err(res.text().await.unwrap_or_default())
        }
    }

    pub fn handle_authenticate(
        pamh: &Pam,
        username: &str,
        svc: PamService,
    ) -> Result<(), PamError> {
        // sys_info(pamh, &format!("RauthyPam - login trying user {username}"));

        let config = Config::load_create(pamh)?;
        //
        // if !full_login
        //     && let Ok(Some(token)) = PamToken::try_load(&config, username)
        //     && token.validate(&config).is_ok()
        // {
        //     return Ok(());
        // }

        // TODO
        let is_remote_user = matches!(svc, PamService::Sudo | PamService::Su);
        // During SSH login, this will be false.
        // Only true AFTER an SSH session ahs been created
        let is_ssh_session = Self::is_remote_session();
        let is_ssh_login = Self::get_service(pamh) == PamService::Ssh;
        let is_remote = is_ssh_login || is_ssh_session;

        sys_info(
            pamh,
            &format!(
                "is_remote_user: {is_remote_user}, is_ssh_login: {is_ssh_login}, is_ssh_session: {is_ssh_session}"
            ),
        );

        let preflight = match RT.block_on(Self::preflight(
            config.rauthy_url.clone(),
            config.host_id.clone(),
            config.host_secret.clone(),
            username.to_string(),
        )) {
            Ok(p) => p,
            Err(err) => {
                sys_err(pamh, &format!("Preflight Error: {err}"));
                return Err(PamError::SERVICE_ERR);
            }
        };
        if !preflight.login_allowed {
            sys_err(pamh, "Login denied for this user");
            return Err(PamError::CRED_INSUFFICIENT);
        }

        let mut login_req = PamLoginRequest {
            host_id: config.host_id.clone(),
            host_secret: config.host_secret.clone(),
            username: username.to_string(),
            password: None,
            remote_password: None,
            webauthn_code: None,
        };

        if preflight.mfa_required && !is_remote {
            match RT.block_on(Self::mfa(config.rauthy_url.clone(), username.to_string())) {
                Ok(webauthn_code) => {
                    login_req.webauthn_code = Some(webauthn_code);
                }
                Err(err) => {
                    sys_err(pamh, &format!("Login Error: {err}"));
                    return Err(PamError::AUTH_ERR);
                }
            }
        } else {
            let text = if is_remote {
                "Remote PAM Password: "
            } else {
                "Password: "
            };
            let password = match pamh.get_authtok(Some(text)) {
                Ok(Some(p)) => p.to_str().unwrap().to_string(),
                Ok(None) => {
                    sys_err(pamh, "No password provided");
                    return Err(PamError::AUTHINFO_UNAVAIL);
                }
                Err(err) => {
                    sys_err(pamh, "Error getting authtok");
                    return Err(err);
                }
            };
            if is_remote {
                login_req.remote_password = Some(password)
            } else {
                login_req.password = Some(password);
            }
        };

        match RT.block_on(Self::send_login(config.rauthy_url.clone(), login_req)) {
            Ok(token) => {
                let msg = if preflight.mfa_required {
                    format!("Rauthy PAM MFA Login successful for user {username}")
                } else {
                    format!("Rauthy PAM Password Login successful for user {username}")
                };
                sys_info(pamh, &msg);

                if let Err(err) = token.save(&config) {
                    sys_err(pamh, &format!("Error saving PAM token: {err}"));
                }

                // token.create_home_dir();

                Ok(())
            }
            Err(err) => {
                sys_err(pamh, &format!("Authentication Error: {err}"));
                Err(PamError::AUTH_ERR)
            }
        }
    }
}
