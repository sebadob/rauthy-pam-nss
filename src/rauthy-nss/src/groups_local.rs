use crate::cache::Cache;
use crate::config::Config;
use crate::error::Error;
use crate::utils::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tokio::fs;

static CACHE_KEY: &str = "$groups_local_all$";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupLocal {
    pub id: u32,
    pub name: String,
    pub members: Vec<String>,
}

impl GroupLocal {
    pub async fn read_id(name: &str) -> Result<Option<Self>, Error> {
        match Self::read().await? {
            None => Ok(None),
            Some(groups) => Ok(groups.get(name).cloned()),
        }
    }

    pub async fn read() -> Result<Option<BTreeMap<String, Self>>, Error> {
        if let Some(cached) = Cache::get(CACHE_KEY.to_string()).await {
            return if let Some(group) = cached {
                let slf = deserialize::<BTreeMap<String, Self>>(&group)?;
                Ok(Some(slf))
            } else {
                Ok(None)
            };
        }

        let groups = Self::read_convert_groups()
            .await?
            .into_iter()
            .map(|g| (g.name.clone(), g))
            .collect::<BTreeMap<String, Self>>();

        Cache::set(
            CACHE_KEY.to_string(),
            Some(serialize(&groups)?),
            Config::get().cache_ttl_groups,
        )
        .await;

        Ok(Some(groups))
    }

    #[inline]
    async fn read_convert_groups() -> Result<Vec<Self>, Error> {
        let s = fs::read_to_string("/etc/group").await?;

        let mut res = Vec::with_capacity(128);
        for line in s.lines() {
            if !line.is_empty()
                && let Some(slf) = Self::maybe_from_str(line)
            {
                res.push(slf);
            }
        }

        Ok(res)
    }

    #[inline]
    fn maybe_from_str(s: &str) -> Option<Self> {
        let mut parts = s.split(':');
        let name = parts.next()?;
        let _x = parts.next()?;
        let id = parts.next()?.parse::<u32>().ok()?;
        let members = parts
            .next()?
            .split(',')
            .map(String::from)
            .collect::<Vec<_>>();

        Some(Self {
            id,
            name: name.to_string(),
            members,
        })
    }
}
