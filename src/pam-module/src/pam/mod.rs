use crate::config::Config;
use crate::constants::{ENV_SESSION, ENV_USER_EMAIL, ENV_USER_ID, ENV_USERNAME};
use crate::pam::token::PamToken;
use pamsm::{LogLvl, Pam, PamError, PamFlags, PamLibExt, PamServiceModule};
use std::sync::OnceLock;
use std::{env, fs};

mod auth;
pub mod token;

static DEBUG: OnceLock<bool> = OnceLock::new();

macro_rules! load_config_token {
    ($pamh:expr, $username:expr) => {{
        let config = match Config::read() {
            Ok(c) => c,
            Err(err) => {
                sys_err($pamh, &format!("Error loading config: {err}"));
                return PamError::SERVICE_ERR;
            }
        };

        let token = match PamToken::try_load(&config, $username) {
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
    Other,
    Unknown,
}

pub struct RauthyPam;

impl RauthyPam {
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
                    "sudo" => PamService::Sudo,
                    "su" => PamService::Su,
                    _ => PamService::Other,
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
        let (config, token) = load_config_token!(&pamh, username);

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
        let (_config, token) = load_config_token!(&pamh, username);

        if let Some(token) = token {
            token.create_home_dir();

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
        let (_config, _token) = load_config_token!(&pamh, username);

        // let _ = get_nonlocal_username!(&pamh);
        // TODO delete token ? Or maybe full logout on server as well?
        // sys_info(&pamh, "in RauthyPam close_session");
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

    let cstr_cached = pamh
        .get_cached_user()
        .unwrap_or_default()
        .unwrap_or_default();
    let sc = cstr_cached.to_str().unwrap_or_default();

    let svc = RauthyPam::get_service(pamh);

    let rhost = pamh.get_rhost().unwrap_or_default().unwrap_or_default();
    let rhost_s = rhost.to_str().unwrap_or_default();

    let ruser = pamh.get_ruser().unwrap_or_default().unwrap_or_default();
    let ruser_s = ruser.to_str().unwrap_or_default();

    sys_info(
        pamh,
        &format!(
            "\n{module} / username {s} / username cached: {sc} / \
            service: {svc:?} / rhost: {rhost_s} / ruser: {ruser_s}"
        ),
    );
    println!(
        "\n{module} / username {s} / username cached: {sc} / \
        service: {svc:?} / rhost: {rhost_s} / ruser: {ruser_s}"
    );
}

#[inline]
fn set_debug(args: &[String]) {
    let _ = DEBUG.set(args.first().map(|v| v.as_str()) == Some("debug"));
}

#[inline]
pub fn sys_err(pamh: &Pam, msg: &str) {
    pamh.syslog(LogLvl::ERR, msg).expect("failed to syslog");
}

#[inline]
fn sys_info(pamh: &Pam, msg: &str) {
    pamh.syslog(LogLvl::INFO, msg).expect("failed to syslog");
}
