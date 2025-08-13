use crate::config::Config;
use crate::{CLIENT, RT, copy_dir};
use chrono::Utc;
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::Permissions;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

static TOKEN: OnceLock<Option<PamToken>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub fn create_home_dir(&self) -> anyhow::Result<()> {
        let path = PathBuf::from("/home").join(&self.username);

        if !fs::exists(&path)? {
            fs::create_dir_all(&path).expect("Cannot create user homedir");

            #[cfg(unix)]
            {
                use std::os::unix::fs::{PermissionsExt, chown};

                chown(&path, Some(self.uid), Some(self.gid))
                    .expect("Cannot set user:group for home dir");
                fs::set_permissions(&path, Permissions::from_mode(0o700))
                    .expect("Cannot set permissions for new user homedir");

                // TODO even though this works fine, it requires an SELinux rule that I don't
                //  want. Is there maybe a nicer solution to this?
                // we want to ignore the result here, because SELinux may not even be installed
                let _ = Command::new("/usr/sbin/restorecon")
                    .arg("-rF")
                    .arg(format!("/home/{}", self.username))
                    .output();
            }

            if let Some(skel) = &Config::read()?.home_dir_skel {
                copy_dir::recursive_copy_dir_all(skel, &path, Some(self.uid), Some(self.gid))?;
            }
        }

        Ok(())
    }

    pub fn try_load(
        config: &Config,
        username: &str,
        with_validation: bool,
    ) -> anyhow::Result<Option<&'static Self>> {
        if let Some(opt) = TOKEN.get() {
            return if let Some(slf) = opt {
                Ok(Some(slf))
            } else {
                Ok(None)
            };
        }

        let base = config.data_path_user(username);
        let path = base.join("token");

        let bytes = fs::read(path)?;
        let (slf, _) =
            bincode::serde::decode_from_slice::<Self, _>(&bytes, bincode::config::standard())?;

        if with_validation {
            match slf.validate(config) {
                Ok(_) => {
                    TOKEN.set(Some(slf)).unwrap();
                    Ok(Some(TOKEN.get().unwrap().as_ref().unwrap()))
                }
                Err(err) => {
                    eprintln!("Token Validation Error: {err}");
                    Ok(None)
                }
            }
        } else {
            TOKEN.set(Some(slf)).unwrap();
            Ok(Some(TOKEN.get().unwrap().as_ref().unwrap()))
        }
    }

    #[inline]
    pub fn save(&self, config: &Config) -> anyhow::Result<()> {
        let base = config.data_path_user(&self.username);
        let path = base.join("token");

        let bytes = bincode::serde::encode_to_vec(self, bincode::config::standard())?;
        fs::write(path, bytes)?;

        Ok(())
    }

    pub fn validate(&self, config: &Config) -> Result<(), String> {
        let now = Utc::now().timestamp();
        if self.exp < now {
            return Err("PAM token has expired".to_string());
        }

        let url = format!("{}auth/v1/pam/validate/{}", config.rauthy_url, self.user_id);
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
