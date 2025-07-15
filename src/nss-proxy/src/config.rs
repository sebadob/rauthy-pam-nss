use log::debug;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fs;
use std::fs::Permissions;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::OnceLock;

#[cfg(debug_assertions)]
static PATH: &str = "./proxy.toml";
#[cfg(not(debug_assertions))]
static PATH: &str = "/etc/rauthy/proxy.toml";

static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogTarget {
    Console,
    File,
    ConsoleFile,
    Syslog,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    // pub listen_addr: String,
    pub rauthy_url: Url,
    pub host_id: String,
    pub host_secret: String,
    #[serde(default = "data_path")]
    pub data_dir: Cow<'static, str>,
    pub log_target: LogTarget,
    pub danger_allow_unsecure: bool,
    pub workers: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // listen_addr: "127.0.0.1:8000".to_string(),
            rauthy_url: "https://iam.example.com".parse().unwrap(),
            host_id: "hostIdFromRauthy".to_string(),
            host_secret: "hostSecretFromRauthy".to_string(),
            data_dir: data_path(),
            log_target: LogTarget::Syslog,
            danger_allow_unsecure: false,
            workers: 1,
        }
    }
}

#[inline]
fn data_path() -> Cow<'static, str> {
    "/var/lib/rauthy_proxy".into()
}

impl Config {
    #[inline]
    pub fn create_data_dir(&self) -> anyhow::Result<()> {
        fs::create_dir_all(self.data_dir.as_ref())?;
        fs::set_permissions(self.data_dir.as_ref(), Permissions::from_mode(0o700))?;
        Ok(())
    }

    #[inline(always)]
    pub fn get() -> &'static Self {
        CONFIG.get().unwrap()
    }

    pub fn load() -> anyhow::Result<()> {
        match Self::read() {
            Ok(slf) => {
                slf.create_data_dir()?;
                CONFIG.set(slf).unwrap();
                Ok(())
            }
            Err(err) => {
                eprintln!("{err}");
                match Self::create_template() {
                    Ok(_) => Err(anyhow::Error::msg(format!(
                        "Creating template config in {PATH}. Edit it and paste the correct values."
                    ))),
                    Err(err) => Err(err),
                }
            }
        }
    }

    #[inline]
    pub fn create_template() -> anyhow::Result<()> {
        if fs::exists(PATH)? {
            debug!("Config file exists already - nothing to do");
            return Ok(());
        }

        let path = PathBuf::from(PATH);
        let parent = path.parent().unwrap();
        fs::create_dir_all(parent)?;
        fs::set_permissions(parent, Permissions::from_mode(0o600))?;

        fs::File::create_new(&path)?;
        fs::set_permissions(path, Permissions::from_mode(0o600))?;

        let slf = Self::default();
        let s = toml::to_string_pretty(&slf)?;

        fs::write(PATH, s)?;

        Ok(())
    }

    #[inline]
    pub fn read() -> anyhow::Result<Self> {
        let mut file = fs::File::open(PATH)?;

        let mut content = String::with_capacity(128);
        file.read_to_string(&mut content)?;

        let slf = toml::from_str::<Self>(&content)?;

        Ok(slf)
    }
}
