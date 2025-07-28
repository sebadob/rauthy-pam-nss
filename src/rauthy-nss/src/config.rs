use log::debug;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::Permissions;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::OnceLock;
use url::Url;

#[cfg(debug_assertions)]
pub static CONFIG_PATH: &str = "./rauthy-pam-nss.toml";
#[cfg(not(debug_assertions))]
pub static CONFIG_PATH: &str = "/etc/rauthy/rauthy-pam-nss.toml";

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
    // #[serde(default = "data_path")]
    // pub data_dir: Cow<'static, str>,
    pub log_target: LogTarget,
    pub danger_allow_insecure: bool,
    pub workers: usize,
    pub cache_ttl_groups: u32,
    pub cache_ttl_hosts: u32,
    pub cache_ttl_users: u32,
    pub cache_flush_interval: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // listen_addr: "127.0.0.1:8000".to_string(),
            rauthy_url: "https://iam.example.com".parse().unwrap(),
            host_id: "hostIdFromRauthy".to_string(),
            host_secret: "hostSecretFromRauthy".to_string(),
            // data_dir: data_path(),
            log_target: LogTarget::Syslog,
            danger_allow_insecure: false,
            workers: 1,
            cache_ttl_groups: 60,
            cache_ttl_hosts: 60,
            cache_ttl_users: 60,
            cache_flush_interval: 900,
        }
    }
}

impl Config {
    #[inline(always)]
    pub fn get() -> &'static Self {
        CONFIG.get().unwrap()
    }

    pub fn load() -> anyhow::Result<()> {
        match Self::read() {
            Ok(slf) => {
                CONFIG.set(slf).unwrap();
                Ok(())
            }
            Err(err) => {
                eprintln!("{err}");
                match Self::create_template() {
                    Ok(_) => Err(anyhow::Error::msg(format!(
                        "Creating template config in {CONFIG_PATH}. Edit it and paste the correct values."
                    ))),
                    Err(err) => Err(err),
                }
            }
        }
    }

    #[inline]
    pub fn create_template() -> anyhow::Result<()> {
        if fs::exists(CONFIG_PATH)? {
            debug!("Config file exists already - nothing to do");
            return Ok(());
        }

        let path = PathBuf::from(CONFIG_PATH);
        let parent = path.parent().unwrap();
        fs::create_dir_all(parent)?;
        fs::set_permissions(parent, Permissions::from_mode(0o600))?;

        fs::File::create_new(&path)?;
        fs::set_permissions(path, Permissions::from_mode(0o600))?;

        let slf = Self::default();
        let s = toml::to_string_pretty(&slf)?;

        fs::write(CONFIG_PATH, s)?;

        Ok(())
    }

    #[inline]
    pub fn read() -> anyhow::Result<Self> {
        let mut file = fs::File::open(CONFIG_PATH)?;

        let mut content = String::with_capacity(128);
        file.read_to_string(&mut content)?;

        let slf = toml::from_str::<Self>(&content)?;

        Ok(slf)
    }
}
