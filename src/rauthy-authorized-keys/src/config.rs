use serde::{Deserialize, Serialize};
use std::fs;

#[cfg(debug_assertions)]
pub static PATH: &str = "./rauthy-pam-nss.toml";
#[cfg(not(debug_assertions))]
pub static PATH: &str = "/etc/rauthy/rauthy-pam-nss.toml";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogTarget {
    Console,
    File,
    ConsoleFile,
    Syslog,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub rauthy_url: String,
    pub host_id: String,
    pub host_secret: String,
    pub log_target: LogTarget,
    #[serde(default = "bool_false")]
    pub danger_allow_insecure: bool,
}

fn bool_false() -> bool {
    false
}

impl Config {
    #[inline]
    pub fn read() -> anyhow::Result<Self> {
        let content = fs::read_to_string(PATH)?;
        let slf = toml::from_str::<Self>(&content)?;
        Ok(slf)
    }
}
