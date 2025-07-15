use log::debug;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use tokio::fs;
use tokio::io::AsyncReadExt;

#[cfg(debug_assertions)]
static PATH: &str = "./proxy.toml";
#[cfg(not(debug_assertions))]
static PATH: &str = "/etc/rauthy/proxy.toml";

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
    pub listen_addr: String,
    pub rauthy_url: Url,
    pub host_id: String,
    pub host_secret: String,
    #[serde(default = "data_path")]
    pub data_dir: Cow<'static, str>,
    pub log_target: LogTarget,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:8000".to_string(),
            rauthy_url: "https://iam.example.com".parse().unwrap(),
            host_id: "hostIdFromRauthy".to_string(),
            host_secret: "hostSecretFromRauthy".to_string(),
            data_dir: data_path(),
            log_target: LogTarget::Syslog,
        }
    }
}

#[inline]
fn data_path() -> Cow<'static, str> {
    "/var/lib/rauthy_proxy".into()
}

impl Config {
    #[inline]
    pub async fn create_data_dir(&self) -> anyhow::Result<()> {
        fs::create_dir_all(self.data_dir.as_ref()).await?;
        fs::set_permissions(self.data_dir.as_ref(), Permissions::from_mode(0o700)).await?;
        Ok(())
    }

    pub async fn load() -> anyhow::Result<Self> {
        match Self::read().await {
            Ok(slf) => {
                slf.create_data_dir().await?;
                Ok(slf)
            }
            Err(err) => {
                eprintln!("{err}");
                match Self::create_template().await {
                    Ok(_) => Err(anyhow::Error::msg(format!(
                        "Creating template config in {PATH}. Edit it and paste the correct values."
                    ))),
                    Err(err) => Err(err),
                }
            }
        }
    }

    #[inline]
    pub async fn create_template() -> anyhow::Result<()> {
        if fs::try_exists(PATH).await? {
            debug!("Config file exists already - nothing to do");
            return Ok(());
        }

        fs::File::create_new(PATH).await?;
        fs::set_permissions(PATH, Permissions::from_mode(0o600)).await?;

        let slf = Self::default();
        let s = toml::to_string_pretty(&slf)?;

        fs::write(PATH, s).await?;

        Ok(())
    }

    #[inline]
    pub async fn read() -> anyhow::Result<Self> {
        let mut file = fs::File::open(PATH).await?;

        let mut content = String::with_capacity(128);
        file.read_to_string(&mut content).await?;

        let slf = toml::from_str::<Self>(&content)?;

        Ok(slf)
    }
}
