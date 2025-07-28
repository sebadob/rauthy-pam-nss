use crate::config::Config;
use crate::constants::{ENV_SESSION, ENV_USER_EMAIL, ENV_USER_ID, ENV_USERNAME};
use crate::pam::token::PamToken;
use pamsm::{LogLvl, Pam, PamError, PamFlags, PamLibExt, PamServiceModule};
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;
use std::{env, fs};

mod auth;
pub mod token;
mod webauthn;

static DEBUG: OnceLock<bool> = OnceLock::new();

macro_rules! load_config_token {
    ($pamh:expr, $username:expr, $validate:expr) => {{
        let config = match Config::read() {
            Ok(c) => c,
            Err(err) => {
                sys_err($pamh, &format!("Error loading config: {err}"));
                return PamError::SERVICE_ERR;
            }
        };

        let token = match PamToken::try_load(&config, $username, $validate) {
            Ok(t) => t,
            Err(err) => {
                sys_err($pamh, &format!("Error loading PAM token: {err}"));
                return PamError::SERVICE_ERR;
            }
        };

        (config, token)
    }};
}

macro_rules! get_nonlocal_username {
    ($pamh:expr) => {{
        let username = match $pamh.get_user(Some("User: ")) {
            Ok(Some(u)) => u.to_str().unwrap(),
            Ok(None) => return PamError::AUTH_ERR,
            Err(err) => return err,
        };
        let username = if username.len() >= 2 {
            username
        } else {
            return PamError::CRED_UNAVAIL;
        };

        if RauthyPam::is_local_user(username) {
            return PamError::CRED_UNAVAIL;
        }

        username
    }};
}

macro_rules! get_nonlocal_r_username {
    ($pamh:expr) => {{
        let username = match $pamh.get_ruser() {
            Ok(Some(u)) => u.to_str().unwrap(),
            Ok(None) => return PamError::AUTH_ERR,
            Err(err) => return err,
        };
        let username = if username.len() >= 2 {
            username
        } else {
            return PamError::CRED_UNAVAIL;
        };

        if RauthyPam::is_local_user(username) {
            return PamError::CRED_UNAVAIL;
        }

        username
    }};
}

#[derive(Debug, PartialEq)]
pub enum PamService {
    Login,
    Ssh,
    Sudo,
    Su,
    Other(String),
    Unknown,
}

pub struct RauthyPam;

impl RauthyPam {
    fn exec_script(pamh: &Pam, path: &PathBuf, token: &PamToken) -> anyhow::Result<()> {
        let file = fs::File::open(path)?;
        let meta = file.metadata()?;
        if !meta.is_file() {
            return Err(anyhow::Error::msg("target path is not a file"));
        }

        #[cfg(unix)]
        {
            use std::os::unix::prelude::*;

            if meta.uid() != 0 {
                return Err(anyhow::Error::msg(format!(
                    "{path:?} must be owned by root"
                )));
            }

            let mode = meta.permissions().mode();
            if mode != 0o100700 {
                return Err(anyhow::Error::msg(format!(
                    "Invalid permissions on script, expected {:#o}, found {:#o}",
                    0o100700, mode
                )));
            }
        }

        let cmd = format!(
            "{} {} {} {} {} {}",
            path.to_str().unwrap_or_default(),
            token.username,
            token.uid,
            token.gid,
            token.user_id,
            token.user_email
        );
        let res = Command::new("/bin/bash").arg("-c").arg(cmd).output()?;

        if res.status.success() {
            if *DEBUG.get().unwrap() {
                let out = String::from_utf8_lossy(res.stdout.as_slice());
                if !out.is_empty() {
                    sys_info(pamh, out.as_ref());
                }
            }
            Ok(())
        } else {
            let err = String::from_utf8_lossy(res.stderr.as_slice());
            sys_err(pamh, err.as_ref());
            Err(anyhow::Error::msg(err.to_string()))
        }
    }

    #[inline]
    fn is_local_user(username: &str) -> bool {
        let passwd = fs::read_to_string("/etc/passwd").expect("Cannot access /etc/passwd");
        let name = format!("{username}:");
        for line in passwd.lines() {
            if line.starts_with(&name) {
                return true;
            }
        }
        false
    }

    #[inline]
    fn is_remote_session() -> bool {
        dotenvy::dotenv().ok();
        env::vars().any(|(k, v)| k == ENV_SESSION && v == "remote")
    }

    #[inline]
    fn get_service(pamh: &Pam) -> PamService {
        match pamh.get_service() {
            Ok(v) => {
                let svc = v.unwrap_or_default().to_str().unwrap_or_default();
                if *DEBUG.get().unwrap() {
                    sys_info(pamh, &format!("Service detected: {svc}"));
                }

                match svc.to_lowercase().as_str() {
                    "login" => PamService::Login,
                    "sshd" => PamService::Ssh,
                    "sudo" | "sudo-i" => PamService::Sudo,
                    "su" | "su-l" => PamService::Su,
                    s => PamService::Other(s.to_string()),
                }
            }
            Err(err) => {
                sys_err(pamh, &format!("Cannot read service: {err:?}"));
                PamService::Unknown
            }
        }
    }
}

impl PamServiceModule for RauthyPam {
    fn acct_mgmt(pamh: Pam, _: PamFlags, args: Vec<String>) -> PamError {
        set_debug(&args);
        debug(&pamh, "acct_mgmt");

        let svc = Self::get_service(&pamh);
        let username = if matches!(svc, PamService::Sudo | PamService::Su) {
            get_nonlocal_r_username!(&pamh)
        } else {
            get_nonlocal_username!(&pamh)
        };
        let (config, token) = load_config_token!(&pamh, username, true);

        if let Some(token) = token
            && token.validate(config).is_ok()
        {
            PamError::SUCCESS
        } else {
            PamError::AUTHINFO_UNAVAIL
        }
    }

    fn authenticate(pamh: Pam, _: PamFlags, args: Vec<String>) -> PamError {
        set_debug(&args);
        debug(&pamh, "authenticate");

        let svc = Self::get_service(&pamh);
        let username = if matches!(svc, PamService::Sudo | PamService::Su) {
            get_nonlocal_r_username!(&pamh)
        } else {
            get_nonlocal_username!(&pamh)
        };
        // println!("authenticate username: {username}");

        match Self::handle_authenticate(&pamh, username, svc) {
            Ok(_) => PamError::SUCCESS,
            Err(err) => {
                sys_err(&pamh, &format!("Rauthy PAM login failed with {err}"));
                err
            }
        }
    }

    // will usually be triggered as root and NOT with the actual user
    fn setcred(pamh: Pam, _: PamFlags, args: Vec<String>) -> PamError {
        set_debug(&args);
        debug(&pamh, "setcred");

        let _username = get_nonlocal_username!(&pamh);
        // let (config, _) = load_config_token!(&pamh, username);
        // println!(
        //     r#"You cannot change your credentials here, please go to your account dashboard:
        //
        //     {}/auth/v1/account
        //     "#,
        //     config.rauthy_url
        // );

        PamError::SUCCESS
    }

    fn open_session(pamh: Pam, _: PamFlags, args: Vec<String>) -> PamError {
        set_debug(&args);
        debug(&pamh, "open_session");

        // TODO will we ever need to check the remote user here?
        let username = get_nonlocal_username!(&pamh);
        let (config, token) = load_config_token!(&pamh, username, false);

        if let Some(token) = token {
            if let Err(err) = token.create_home_dir() {
                sys_err(&pamh, &err.to_string());
            }

            let session_typ = if Self::get_service(&pamh) == PamService::Ssh {
                "remote"
            } else {
                "local"
            };
            pamh.putenv(&format!("{ENV_SESSION}={session_typ}"))
                .unwrap();
            pamh.putenv(&format!("{ENV_USER_ID}={}", token.user_id))
                .unwrap();
            pamh.putenv(&format!("{ENV_USER_EMAIL}={}", token.user_email))
                .unwrap();
            pamh.putenv(&format!("{ENV_USERNAME}={}", token.username))
                .unwrap();

            if let Some(path) = &config.exec_session_open {
                let svc = Self::get_service(&pamh);

                if (svc == PamService::Login || svc == PamService::Ssh)
                    && let Err(err) = Self::exec_script(&pamh, path, token)
                {
                    sys_err(&pamh, &err.to_string());
                }
            }

            PamError::SUCCESS
        } else {
            eprintln!("No token in open session");
            PamError::AUTHINFO_UNAVAIL
        }
    }

    // will usually be triggered as root and NOT with the actual user
    fn close_session(pamh: Pam, _flags: PamFlags, args: Vec<String>) -> PamError {
        set_debug(&args);
        debug(&pamh, "close_session");

        let username = get_nonlocal_username!(&pamh);
        // let username = get_nonlocal_r_username!(&pamh);
        let (config, token) = load_config_token!(&pamh, username, false);

        // let _ = get_nonlocal_username!(&pamh);
        // TODO delete token ? Or maybe full logout on server as well?
        // sys_info(&pamh, "in RauthyPam close_session");

        if let Some(path) = &config.exec_session_close {
            match token {
                None => {
                    sys_err(
                        &pamh,
                        "No PamToken found - cannot execute session close script",
                    );
                }
                Some(token) => {
                    let svc = Self::get_service(&pamh);
                    if (svc == PamService::Login || svc == PamService::Ssh)
                        && let Err(err) = Self::exec_script(&pamh, path, token)
                    {
                        sys_err(&pamh, &err.to_string());
                    }
                }
            }
        }

        PamError::SUCCESS
    }
}

fn debug(pamh: &Pam, module: &str) {
    if !*DEBUG.get().unwrap() {
        return;
    }

    let cstr = pamh
        .get_user(Some("User: "))
        .unwrap_or_default()
        .unwrap_or_default();
    let s = cstr.to_str().unwrap_or_default();

    let svc = RauthyPam::get_service(pamh);

    let rhost = pamh.get_rhost().unwrap_or_default().unwrap_or_default();
    let rhost_s = rhost.to_str().unwrap_or_default();

    let ruser = pamh.get_ruser().unwrap_or_default().unwrap_or_default();
    let ruser_s = ruser.to_str().unwrap_or_default();

    sys_info(
        pamh,
        &format!(
            "{module} : username {s} / service: {svc:?} / rhost: {rhost_s} / ruser: {ruser_s}"
        ),
    );
    println!("{module} : username {s} / service: {svc:?} / rhost: {rhost_s} / ruser: {ruser_s}");
}

#[inline]
fn set_debug(args: &[String]) {
    let _ = DEBUG.set(args.iter().any(|v| v.as_str() == "debug"));
}

#[inline]
pub fn sys_err(pamh: &Pam, msg: &str) {
    pamh.syslog(LogLvl::ERR, msg).expect("failed to syslog");
}

#[inline]
fn sys_info(pamh: &Pam, msg: &str) {
    pamh.syslog(LogLvl::INFO, msg).expect("failed to syslog");
}
