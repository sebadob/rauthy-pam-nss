use crate::api_types::Getent;
use crate::cache::Cache;
use crate::config::Config;
use crate::error::{Error, ErrorType};
use crate::http_client::HttpClient;
use crate::{RAUTHY_HEALTHY, VERSION};
use axum::body::Body;
use axum::http::Response;
use log::{debug, error};
use std::sync::atomic::Ordering;

pub mod groups;
pub mod hosts;
pub mod users;

pub type ApiResponse = Result<Response<Body>, Error>;

static CACHE_KEY_USERS: &str = "$users$";
static CACHE_KEY_GROUPS: &str = "$groups$";
static CACHE_KEY_HOSTS: &str = "$hosts$";

pub async fn get_root() -> String {
    format!("Rauthy NSS Proxy v{VERSION}")
}

async fn fetch_getent(getent: Getent) -> ApiResponse {
    let (cached, cache_key) = match &getent {
        Getent::Users => {
            let cached = Cache::get(CACHE_KEY_USERS.to_string()).await;
            (cached, CACHE_KEY_USERS.to_string())
        }
        Getent::Username(username) => {
            let key = format!("u_{username}");
            let cached = Cache::get(key.clone()).await;
            (cached, key)
        }
        Getent::UserId(uid) => {
            let key = format!("u_{uid}");
            let cached = Cache::get(key.clone()).await;
            (cached, key)
        }
        Getent::Groups => {
            let cached = Cache::get(CACHE_KEY_GROUPS.to_string()).await;
            (cached, CACHE_KEY_GROUPS.to_string())
        }
        Getent::Groupname(groupname) => {
            let key = format!("g_{groupname}");
            let cached = Cache::get(key.to_string()).await;
            (cached, key)
        }
        Getent::GroupId(gid) => {
            let key = format!("g_{gid}");
            let cached = Cache::get(key.to_string()).await;
            (cached, key)
        }
        Getent::Hosts => {
            let cached = Cache::get(CACHE_KEY_HOSTS.to_string()).await;
            (cached, CACHE_KEY_HOSTS.to_string())
        }
        Getent::Hostname(hostname) => {
            let key = format!("h_{hostname}");
            let cached = Cache::get(key.to_string()).await;
            (cached, key)
        }
        Getent::HostIp(ip) => {
            let key = format!("h_{ip}");
            let cached = Cache::get(key.to_string()).await;
            (cached, key)
        }
    };

    if let Some(opt) = cached {
        debug!("Cache hit");
        return match opt {
            None => {
                // we do this check for negative caching
                Err(Error::new(ErrorType::NotFound, "value not found"))
            }
            Some(value) => Ok(Response::builder()
                .status(200)
                .body(Body::from(value))
                .unwrap()),
        };
    }

    if !RAUTHY_HEALTHY.load(Ordering::Relaxed) {
        // We must avoid a chicken-and-egg problem here.
        // During startup for instance, we try to resolve our own target
        // address via the systems hosts, for which we should provide the data.
        // If the connection to Rauthy is unhealthy or Rauthy itself is unhealthy,
        // just send back NotFound.
        return Err(Error::new(ErrorType::NotFound, "value not found"));
    }

    let config = Config::get();

    let resp = match HttpClient::getent(&getent).await {
        Ok(r) => r,
        Err(err) => {
            error!("Rauthy Connection Error: {err:?}");
            // Cache::set(cache_key, None, 10);
            return Err(Error::new(ErrorType::NotFound, "value not found"));
        }
    };

    let value = resp.clone();
    let ttl = match getent {
        Getent::Users | Getent::UserId(_) | Getent::Username(_) => config.cache_ttl_users,
        Getent::Groups | Getent::GroupId(_) | Getent::Groupname(_) => config.cache_ttl_groups,
        Getent::Hosts | Getent::Hostname(_) | Getent::HostIp(_) => config.cache_ttl_hosts,
    };
    Cache::set(cache_key, value, ttl).await;

    match resp {
        None => Err(Error::new(ErrorType::NotFound, "value not found")),
        Some(value) => Ok(Response::builder()
            .status(200)
            .body(Body::from(value))
            .unwrap()),
    }
}
