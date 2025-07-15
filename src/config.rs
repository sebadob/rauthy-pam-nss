use crate::pam::sys_err;
use pamsm::{Pam, PamError};
use reqwest::Url;
use serde::Deserialize;
use std::borrow::Cow;
use std::fs;
use std::fs::{File, Permissions};
use std::io::Read;
use std::os::unix::fs::PermissionsExt;

static PATH: &str = "/etc/security/pam_rauthy.toml";

#[derive(Debug, Deserialize)]
pub struct Config {
    pub url: Url,
    pub host_id: String,
    pub host_secret: String,
    #[serde(default = "data_path")]
    pub data_path: Cow<'static, str>,
    // pub wheel_roles: Vec<String>,
    // pub wheel_groups: Vec<String>,
}

#[inline]
fn data_path() -> Cow<'static, str> {
    "/var/lib/pam_rauthy".into()
}

impl Config {
    #[inline]
    pub fn data_path_user(&self, username: &str) -> String {
        let path = format!("{}/{}", self.data_path, username);

        // these should never panic, when we were able to create and parse config beforehand
        fs::create_dir_all(&path).expect("Cannot create user data dir");
        fs::set_permissions(&path, Permissions::from_mode(0o600))
            .expect("Cannot set permissions on user data dir");

        path
    }

    pub fn load(pamh: &Pam) -> Result<Self, PamError> {
        match Self::read() {
            Ok(slf) => Ok(slf),
            Err(err) => {
                sys_err(pamh, &format!("Error loading config file: {err}"));
                Err(PamError::SERVICE_ERR)
            }
        }
    }

    #[inline]
    pub fn url_getent(&self) -> String {
        format!("{}auth/v1/pam/getent", self.url)
    }

    #[inline]
    pub fn read() -> anyhow::Result<Self> {
        let mut file = File::open(PATH)?;
        fs::set_permissions(PATH, Permissions::from_mode(0o600))?;

        let mut content = String::with_capacity(128);
        file.read_to_string(&mut content)?;

        let slf = toml::from_str::<Self>(&content)?;

        // make sure data path exists and perms are correct
        fs::create_dir_all(slf.data_path.as_ref())?;
        fs::set_permissions(slf.data_path.as_ref(), Permissions::from_mode(0o600))?;

        Ok(slf)
    }
}
