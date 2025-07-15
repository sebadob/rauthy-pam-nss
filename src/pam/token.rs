use crate::config::Config;
use crate::{CLIENT, RT};
use chrono::Utc;
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::Permissions;
use std::os::unix::fs::{PermissionsExt, chown};

#[derive(Debug, Serialize, Deserialize)]
pub struct PamToken {
    pub id: String,
    pub exp: i64,
    pub user_id: String,
    pub user_email: String,
    pub uid: u32,
    pub gid: u32,
    pub username: String,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
}

impl PamToken {
    pub fn create_home_dir(&self) {
        let path = format!("/home/{}", self.username);
        fs::create_dir_all(&path).expect("Cannot create user homedir");
        chown(&path, Some(self.uid), Some(self.gid)).expect("Cannot set user:group for home dir");
        fs::set_permissions(&path, Permissions::from_mode(0o700))
            .expect("Cannot set permissions for new user homedir");
    }

    pub fn try_load(config: &Config, username: &str) -> anyhow::Result<Option<Self>> {
        let base = config.data_path_user(username);
        let path = format!("{base}/token");

        let bytes = fs::read(path)?;
        let (slf, _) =
            bincode::serde::decode_from_slice::<Self, _>(&bytes, bincode::config::standard())?;

        match slf.validate(config) {
            Ok(_) => Ok(Some(slf)),
            Err(err) => {
                eprintln!("Token Validation Error: {err}");
                Ok(None)
            }
        }
    }

    #[inline]
    pub fn save(&self, config: &Config) -> anyhow::Result<()> {
        let base = config.data_path_user(&self.username);
        let path = format!("{base}/token");

        let bytes = bincode::serde::encode_to_vec(self, bincode::config::standard())?;
        fs::write(path, bytes)?;

        Ok(())
    }

    pub fn validate(&self, config: &Config) -> Result<(), String> {
        let now = Utc::now().timestamp();
        if self.exp < now {
            return Err("PAM token has expired".to_string());
        }

        let url = format!("{}auth/v1/pam/validate/{}", config.url, self.user_id);
        RT.block_on(async move {
            let res = CLIENT
                .get(url)
                .header(AUTHORIZATION, format!("PamToken {}", self.id))
                .send()
                .await
                .map_err(|err| err.to_string())?;

            if res.status().is_success() {
                Ok(())
            } else {
                Err(res.text().await.unwrap_or_default())
            }
        })
    }
}
