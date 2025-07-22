use crate::api_types::{Getent, GetentResponse, PamGroupType};
use crate::cache::Cache;
use crate::config::Config;
use crate::error::{Error, ErrorType};
use crate::groups_local::GroupLocal;
use crate::http_client::HttpClient;
use crate::utils::serialize;
use crate::{RAUTHY_HEALTHY, VERSION};
use axum::body::Body;
use axum::http::Response;
use log::{debug, error, info, warn};
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
            Cache::set(cache_key, None, 10).await;
            return Err(Error::new(ErrorType::NotFound, "value not found"));
        }
    };

    let bytes = if let Some(resp) = resp {
        let resp = match resp {
            GetentResponse::Users(r) => GetentResponse::Users(r),
            GetentResponse::User(r) => GetentResponse::User(r),
            GetentResponse::Groups(mut groups) => {
                if let Some(locals) = GroupLocal::read().await? {
                    for group in groups.iter_mut() {
                        if group.typ == PamGroupType::Local {
                            match locals.get(&group.name) {
                                None => {
                                    warn!("Local Group {} not found", group.name);
                                }
                                Some(local) => {
                                    info!(
                                        "Local Group {} found with id {} - replacing it",
                                        group.name, local.id
                                    );
                                    group.id = local.id;
                                }
                            }
                        }
                    }
                }
                GetentResponse::Groups(groups)
            }
            GetentResponse::Group(mut group) => {
                if group.typ == PamGroupType::Local
                    && let Some(local) = GroupLocal::read_id(&group.name).await?
                {
                    group.id = local.id;
                }
                GetentResponse::Group(group)
            }
            GetentResponse::Hosts(g) => GetentResponse::Hosts(g),
            GetentResponse::Host(g) => GetentResponse::Host(g),
        };

        Some(serialize(&resp)?)
    } else {
        None
    };

    // TODO in case we fetched a group, we need to deserialize the response and check if we need
    //  to do a local group mapping
    let ttl = match getent {
        Getent::Users | Getent::UserId(_) | Getent::Username(_) => config.cache_ttl_users,
        Getent::Groups | Getent::GroupId(_) | Getent::Groupname(_) => config.cache_ttl_groups,
        Getent::Hosts | Getent::Hostname(_) | Getent::HostIp(_) => config.cache_ttl_hosts,
    };
    Cache::set(cache_key, bytes.clone(), ttl).await;

    match bytes {
        None => Err(Error::new(ErrorType::NotFound, "value not found")),
        Some(value) => Ok(Response::builder()
            .status(200)
            .body(Body::from(value))
            .unwrap()),
    }
}
