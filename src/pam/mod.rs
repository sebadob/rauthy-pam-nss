use crate::config::Config;
use crate::pam::token::PamToken;
use pamsm::{LogLvl, Pam, PamError, PamFlags, PamLibExt, PamServiceModule};
use std::fs;

mod auth;
pub mod token;

macro_rules! load_config_token {
    ($pamh:expr, $username:expr) => {{
        let config = match Config::load($pamh) {
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

// macro_rules! get_nonlocal_r_username {
//     ($pamh:expr) => {{
//         let username = match $pamh.get_ruser() {
//             Ok(Some(u)) => u.to_str().unwrap(),
//             Ok(None) => return PamError::AUTH_ERR,
//             Err(err) => return err,
//         };
//         let username = if username.len() >= 2 {
//             username
//         } else {
//             return PamError::CRED_UNAVAIL;
//         };
//
//         if RauthyPam::is_local_user(username) {
//             return PamError::CRED_UNAVAIL;
//         }
//
//         username
//     }};
// }

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
}

impl PamServiceModule for RauthyPam {
    fn acct_mgmt(pamh: Pam, _: PamFlags, _: Vec<String>) -> PamError {
        debug(&pamh, "acct_mgmt");

        let username = get_nonlocal_username!(&pamh);
        let (config, token) = load_config_token!(&pamh, username);
        // println!("token {token:?}");

        if let Some(token) = token
            && token.validate(&config).is_ok()
        {
            PamError::SUCCESS
        } else {
            PamError::AUTHINFO_UNAVAIL
        }
    }

    fn authenticate(pamh: Pam, _: PamFlags, _: Vec<String>) -> PamError {
        let username = get_nonlocal_username!(&pamh);
        println!("authenticate username: {username}");

        debug(&pamh, "authenticate");

        match Self::handle_authenticate(&pamh, username) {
            Ok(_) => PamError::SUCCESS,
            Err(err) => {
                sys_err(&pamh, &format!("Rauthy PAM login failed with {err}"));
                err
            }
        }
    }

    // will usually be triggered as root and NOT with the actual user
    fn setcred(pamh: Pam, _: PamFlags, _: Vec<String>) -> PamError {
        debug(&pamh, "setcred");

        // TODO should we even do anything here other than making sure it's not a local user?
        //  Can we maybe log an information with the link to the account dashboard?
        let _username = get_nonlocal_username!(&pamh);
        // let (config, token) = load_config_token!(&pamh, username);

        PamError::SUCCESS
    }

    // will usually be triggered as root and NOT with the actual user
    //
    // TODO find a way to retrieve the original username here
    //  -> tricky, because usually root opens and closes sessions
    fn open_session(pamh: Pam, _: PamFlags, _args: Vec<String>) -> PamError {
        debug(&pamh, "open_session");
        // let username = get_nonlocal_username!(&pamh);
        // println!("open_session username: {username}");

        let username = get_nonlocal_username!(&pamh);
        // let username = get_nonlocal_r_username!(&pamh);
        let (_config, token) = load_config_token!(&pamh, username);

        // TODO accept an args to live-validate the token
        if let Some(token) = token {
            token.create_home_dir();
            PamError::SUCCESS
        } else {
            eprintln!("No token in open session");
            PamError::AUTHINFO_UNAVAIL
        }
    }

    // will usually be triggered as root and NOT with the actual user
    fn close_session(pamh: Pam, _flags: PamFlags, _: Vec<String>) -> PamError {
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

    let svc = pamh.get_service().unwrap_or_default().unwrap_or_default();
    let svc_s = svc.to_str().unwrap_or_default();

    let rhost = pamh.get_rhost().unwrap_or_default().unwrap_or_default();
    let rhost_s = rhost.to_str().unwrap_or_default();

    let ruser = pamh.get_ruser().unwrap_or_default().unwrap_or_default();
    let ruser_s = ruser.to_str().unwrap_or_default();

    println!(
        "\n{module} / username {s} / username cached: {sc} / service: {svc_s} / \
        rhost: {rhost_s} / ruser: {ruser_s}"
    );
}

#[inline]
pub fn sys_err(pamh: &Pam, msg: &str) {
    pamh.syslog(LogLvl::ERR, msg).expect("failed to syslog");
}

#[inline]
fn sys_info(pamh: &Pam, msg: &str) {
    pamh.syslog(LogLvl::INFO, msg).expect("failed to syslog");
}
