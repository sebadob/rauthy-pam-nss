use crate::config::Config;
use chrono::Utc;
use dashmap::DashMap;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::{task, time};

static CACHE: LazyLock<DashMap<String, CacheValue>> = LazyLock::new(|| DashMap::with_capacity(128));

#[derive(Debug)]
pub struct CacheValue {
    pub exp: i64,
    // we are saving Options to impl negative caching
    pub value: Option<Vec<u8>>,
}

pub struct Cache;

impl Cache {
    pub fn init() {
        task::spawn(async {
            let mut interval =
                time::interval(Duration::from_secs(Config::get().cache_flush_interval));
            loop {
                interval.tick().await;
                Self::flush();
            }
        });
    }

    #[inline]
    fn flush() {
        let now = Utc::now().timestamp();
        for entry in CACHE.iter() {
            if entry.exp > now {
                CACHE.remove(entry.key());
            }
        }
    }

    #[inline]
    pub fn get(key: &str) -> Option<Vec<u8>> {
        let v = CACHE.get(key)?;
        if v.exp > Utc::now().timestamp() {
            return v.value.clone();
        }
        None
    }

    #[inline]
    pub fn set(key: String, value: Option<Vec<u8>>, ttl: u32) {
        CACHE.insert(
            key,
            CacheValue {
                exp: Utc::now().timestamp() + ttl as i64,
                value,
            },
        );
    }
}
