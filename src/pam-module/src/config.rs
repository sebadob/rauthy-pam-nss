use crate::pam::sys_err;
use pamsm::{Pam, PamError};
use reqwest::Url;
use serde::Deserialize;
use std::fs;
use std::fs::{File, Permissions};
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::OnceLock;

// TODO change path + data to the same location as the proxy to re-use it
static PATH: &str = "/etc/rauthy/rauthy-pam-nss.toml";
// static PATH: &str = "/etc/security/pam_rauthy.toml";

pub static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug, Deserialize)]
pub struct Config {
    pub rauthy_url: Url,
    pub host_id: String,
    pub host_secret: String,
    #[serde(default = "data_path")]
    pub data_path: PathBuf,
    pub home_dir_skel: Option<PathBuf>,
    pub exec_session_open: Option<PathBuf>,
    pub exec_session_close: Option<PathBuf>,
}

#[inline]
fn data_path() -> PathBuf {
    "/var/lib/pam_rauthy".into()
}

impl Config {
    #[inline]
    pub fn data_path_user(&self, username: &str) -> PathBuf {
        let path = self.data_path.join(username);

        // these should never panic, when we were able to create and parse config beforehand
        fs::create_dir_all(&path).expect("Cannot create user data dir");
        fs::set_permissions(&path, Permissions::from_mode(0o600))
            .expect("Cannot set permissions on user data dir");

        path
    }

    pub fn load_create(pamh: &Pam) -> Result<Self, PamError> {
        match Self::read_create() {
            Ok(slf) => Ok(slf),
            Err(err) => {
                sys_err(pamh, &format!("Error loading config file: {err}"));
                Err(PamError::SERVICE_ERR)
            }
        }
    }

    #[inline]
    pub fn read() -> anyhow::Result<&'static Self> {
        if let Some(slf) = CONFIG.get() {
            return Ok(slf);
        }

        let content = fs::read_to_string(PATH)?;
        let slf = toml::from_str::<Self>(&content)?;

        let _ = CONFIG.set(slf);

        Ok(CONFIG.get().unwrap())
    }

    #[inline]
    pub fn read_create() -> anyhow::Result<Self> {
        // println!("config path {PATH}");
        let mut file = File::open(PATH)?;
        // println!("{file:?}");

        let perms = Permissions::from_mode(0o644);
        if file.metadata()?.permissions() != perms {
            fs::set_permissions(PATH, perms)?;
        }

        let mut content = String::with_capacity(128);
        file.read_to_string(&mut content)?;

        let slf = toml::from_str::<Self>(&content)?;

        // make sure data path exists and perms are correct
        if !fs::exists(&slf.data_path)? {
            fs::create_dir_all(&slf.data_path)?;
            fs::set_permissions(&slf.data_path, Permissions::from_mode(0o700))?;
        }

        Ok(slf)
    }
}
