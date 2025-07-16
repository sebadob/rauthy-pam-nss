use crate::VERSION;
use crate::api_types::Getent;
use crate::cache::Cache;
use crate::config::Config;
use crate::error::{Error, ErrorType};
use crate::http_client::HttpClient;
use axum::body::Body;
use axum::http::Response;
use log::{debug, info};

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
            let cached = Cache::get(CACHE_KEY_USERS);
            (cached, CACHE_KEY_USERS.to_string())
        }
        Getent::Username(username) => {
            let key = format!("u_{username}");
            let cached = Cache::get(&key);
            (cached, key)
        }
        Getent::UserId(uid) => {
            let key = format!("u_{uid}");
            let cached = Cache::get(&key);
            (cached, key)
        }
        Getent::Groups => {
            let cached = Cache::get(CACHE_KEY_GROUPS);
            (cached, CACHE_KEY_GROUPS.to_string())
        }
        Getent::Groupname(groupname) => {
            let key = format!("g_{groupname}");
            let cached = Cache::get(&key);
            (cached, key)
        }
        Getent::GroupId(gid) => {
            let key = format!("g_{gid}");
            let cached = Cache::get(&key);
            (cached, key)
        }
        Getent::Hosts => {
            let cached = Cache::get(CACHE_KEY_HOSTS);
            (cached, CACHE_KEY_HOSTS.to_string())
        }
        Getent::Hostname(hostname) => {
            let key = format!("h_{hostname}");
            let cached = Cache::get(&key);
            (cached, key)
        }
        Getent::HostIp(ip) => {
            let key = format!("h_{ip}");
            let cached = Cache::get(&key);
            (cached, key)
        }
    };

    if let Some(opt) = cached {
        debug!("Cache hit");
        return match opt {
            None => {
                // we do this check for negative caching
                Err(Error::new(ErrorType::BadRequest, "value not found"))
            }
            Some(value) => Ok(Response::builder()
                .status(200)
                .body(Body::from(value))
                .unwrap()),
        };
    }

    let resp = HttpClient::getent(&getent).await?;
    info!("Cache miss");

    let config = Config::get();
    let value = resp.clone();
    match getent {
        Getent::Users | Getent::UserId(_) | Getent::Username(_) => {
            Cache::set(cache_key, value, config.cache_ttl_users)
        }
        Getent::Groups | Getent::GroupId(_) | Getent::Groupname(_) => {
            Cache::set(cache_key, value, config.cache_ttl_groups)
        }
        Getent::Hosts | Getent::Hostname(_) | Getent::HostIp(_) => {
            Cache::set(cache_key, value, config.cache_ttl_hosts)
        }
    };

    match resp {
        None => Err(Error::new(ErrorType::BadRequest, "value not found")),
        Some(value) => Ok(Response::builder()
            .status(200)
            .body(Body::from(value))
            .unwrap()),
    }
}
