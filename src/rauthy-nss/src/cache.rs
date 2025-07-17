use crate::config::Config;
use chrono::Utc;
use log::info;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::{task, time};

static TX: OnceLock<flume::Sender<CacheReq>> = OnceLock::new();

#[derive(Debug)]
pub struct CacheValue {
    pub exp: i64,
    // we are saving Options to impl negative caching
    pub value: Option<Vec<u8>>,
}

#[derive(Debug)]
pub enum CacheReq {
    Get {
        ack: oneshot::Sender<Option<Option<Vec<u8>>>>,
        key: String,
    },
    Put {
        key: String,
        value: Option<Vec<u8>>,
        ttl: u32,
    },
    Flush,
}

pub struct Cache;

impl Cache {
    pub fn init() {
        let (tx, rx) = flume::bounded(10);
        TX.set(tx).unwrap();
        thread::spawn(move || Self::handler(rx));
    }

    pub fn spawn_ticker() {
        task::spawn(async move {
            let mut interval =
                time::interval(Duration::from_secs(Config::get().cache_flush_interval));
            loop {
                interval.tick().await;
                let _ = TX.get().unwrap().send_async(CacheReq::Flush).await;
            }
        });
    }

    fn handler(rx: flume::Receiver<CacheReq>) {
        let mut data: HashMap<String, CacheValue> = HashMap::with_capacity(128);

        while let Ok(req) = rx.recv() {
            match req {
                CacheReq::Get { ack, key } => match data.get(&key) {
                    None => {
                        ack.send(None).unwrap();
                    }
                    Some(v) => {
                        if v.exp > Utc::now().timestamp() {
                            ack.send(Some(v.value.clone())).unwrap();
                        } else {
                            ack.send(None).unwrap();
                        }
                    }
                },
                CacheReq::Put { key, value, ttl } => {
                    data.insert(
                        key,
                        CacheValue {
                            exp: Utc::now().timestamp() + ttl as i64,
                            value,
                        },
                    );
                }
                CacheReq::Flush => {
                    let now = Utc::now().timestamp();

                    let remove = data
                        .iter()
                        .filter_map(|(k, v)| if v.exp > now { Some(k.clone()) } else { None })
                        .collect::<Vec<_>>();

                    for key in remove {
                        data.remove(&key);
                    }
                }
            }
        }
        info!("Cache exiting");
    }

    #[inline]
    pub async fn get(key: String) -> Option<Option<Vec<u8>>> {
        let (ack, rx) = oneshot::channel();

        TX.get()
            .unwrap()
            .send_async(CacheReq::Get { ack, key })
            .await
            .ok()?;

        rx.await.ok()?
    }

    #[inline]
    pub async fn set(key: String, value: Option<Vec<u8>>, ttl: u32) {
        let _ = TX
            .get()
            .unwrap()
            .send_async(CacheReq::Put { key, value, ttl })
            .await;
    }
}
