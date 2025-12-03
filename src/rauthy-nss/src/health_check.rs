use crate::RAUTHY_HEALTHY;
use crate::config::Config;
use crate::http_client::HttpClient;
use log::{error, info, warn};
use serde::Deserialize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::{task, time};

#[derive(Debug, Deserialize)]
struct HealthResponse {
    db_healthy: bool,
    cache_healthy: bool,
}

pub fn spawn_health_checker() {
    task::spawn(async {
        loop {
            let currently_healthy = RAUTHY_HEALTHY.load(Ordering::Relaxed);

            if currently_healthy {
                time::sleep(Duration::from_secs(
                    Config::get().health_check_interval_healthy,
                ))
                .await;
            } else {
                time::sleep(Duration::from_secs(
                    Config::get().health_check_interval_unhealthy,
                ))
                .await;
            }

            info!("Running health check");
            if is_rauthy_healthy().await {
                if !currently_healthy {
                    info!("Connection to Rauthy successful and Rauthy is healthy");
                    RAUTHY_HEALTHY.store(true, Ordering::Relaxed);
                }
            } else {
                warn!("Cannot connect to Rauthy or it's unhealthy");
                if currently_healthy {
                    RAUTHY_HEALTHY.store(false, Ordering::Relaxed);
                }
            }
        }
    });
}

pub async fn wait_until_healthy() {
    info!("Waiting until healthy connection to Rauthy");

    loop {
        if is_rauthy_healthy().await {
            info!("Connection to Rauthy successful and Rauthy is healthy");
            RAUTHY_HEALTHY.store(true, Ordering::Relaxed);
            return;
        }
        warn!("Cannot connect to Rauthy or it's unhealthy");
        time::sleep(Duration::from_secs(5)).await;
    }
}

#[inline]
pub async fn is_rauthy_healthy() -> bool {
    let config = Config::get();
    let url = format!("{}auth/v1/health", config.rauthy_url);

    let res = match HttpClient::client().get(url).send().await {
        Ok(r) => r,
        Err(err) => {
            error!("Error sending request to Rauthy: {err:?}");
            return false;
        }
    };

    if res.status().is_success() {
        match res.json::<HealthResponse>().await {
            Ok(health) => health.db_healthy && health.cache_healthy,
            Err(err) => {
                error!("HealthResponse deserialization error: {err:?}");
                false
            }
        }
    } else {
        false
    }
}
