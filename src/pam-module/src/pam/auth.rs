use crate::api_types::{
    PamLoginRequest, PamMfaFinishRequest, PamMfaStartRequest, PamPreflightRequest,
    PamPreflightResponse, WebauthnAuthFinishRequest, WebauthnAuthStartResponse, WebauthnServiceReq,
};
use crate::config::Config;
use crate::pam::token::PamToken;
use crate::pam::webauthn::PamWebauthn;
use crate::pam::{PamService, RauthyPam, sys_err, sys_info};
use crate::{CLIENT, RT};
use pamsm::{Pam, PamError, PamLibExt};
use reqwest::Url;
use std::ops::Deref;
use std::time::Duration;
use tokio::time;

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

    async fn mfa(pamh: &Pam, origin: Url, username: String) -> Result<String, String> {
        println!("Provide your Passkey");

        // This short sleep will fight a race condition.
        // The println may not appear if we try to find keys too quickly, when no key is inserted.
        time::sleep(Duration::from_millis(100)).await;

        let (ui, _) = crate::pam::webauthn::UI.deref();
        let authenticator =
            tokio::time::timeout(Duration::from_secs(20), PamWebauthn::wait_for_passkey(ui))
                .await
                .map_err(|_| "timed out".to_string())?;

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

        match PamWebauthn::perform_auth(pamh, authenticator, origin, resp.rcr.public_key).await {
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
                sys_err(pamh, &format!("Passkey validation error: {err:?}"));
                eprintln!("Passkey validation error: {err:?}");
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
        let config = Config::load_create(pamh)?;
        let is_ssh = Self::is_remote_session() || svc == PamService::Ssh;

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
            sys_err(pamh, &format!("Login denied for user {username}"));
            return Err(PamError::CRED_INSUFFICIENT);
        }
        let is_local_password_only =
            matches!(svc, PamService::Login | PamService::Sudo | PamService::Su)
                && preflight.local_password_only;

        let mut login_req = PamLoginRequest {
            host_id: config.host_id.clone(),
            host_secret: config.host_secret.clone(),
            username: username.to_string(),
            password: None,
            remote_password: None,
            webauthn_code: None,
        };

        if preflight.mfa_required && !is_ssh && !is_local_password_only {
            match RT.block_on(Self::mfa(
                pamh,
                config.rauthy_url.clone(),
                username.to_string(),
            )) {
                Ok(webauthn_code) => {
                    login_req.webauthn_code = Some(webauthn_code);
                }
                Err(err) => {
                    sys_err(pamh, &format!("Login Error: {err}"));
                    return Err(PamError::AUTH_ERR);
                }
            }
        } else {
            let text = if is_ssh {
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
            if is_ssh {
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

                // Note: We are creating the home dir during login and not in session, where it
                // would make more sense from a logical standpoint. The reason is that we can have
                // more hardened SELinux rules, if we do it here. During the session creation,
                // we are usually executing from an `unconfined_t` context, where we definitely
                // do not want to allow relabeling files.
                if let Err(err) = token.create_home_dir() {
                    sys_err(pamh, &err.to_string());
                }

                if let Err(err) = token.save(&config) {
                    sys_err(pamh, &format!("Error saving PAM token: {err}"));
                }

                Ok(())
            }
            Err(err) => {
                sys_err(pamh, &format!("Authentication Error: {err}"));
                Err(PamError::AUTH_ERR)
            }
        }
    }
}
