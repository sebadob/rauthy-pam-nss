mod group;
mod hosts;
mod passwd;
mod shadow;

#[macro_export]
macro_rules! load_config_nss {
    () => {
        match $crate::config::Config::read() {
            Ok(c) => c,
            Err(err) => {
                log::error!("Error loading config: {}", err);
                return libnss::interop::Response::Unavail;
            }
        }
    };
}

#[macro_export]
macro_rules! send_getent {
    ($url: expr, $payload: expr) => {
        match $crate::RT.block_on(async {
            let res = match $crate::CLIENT.get($url).json(&$payload).send().await {
                Ok(res) => res,
                Err(err) => {
                    if err.is_connect() {
                        log::error!("Connection error sending getent request: {}", err);
                    } else if err.is_timeout() {
                        log::error!("Timeout error sending getent request: {}", err);
                    } else {
                        log::error!("Error sending getent request: {}", err);
                    }
                    return Err(libnss::interop::Response::TryAgain);
                }
            };

            if res.status().is_success() {
                match res.json::<$crate::api_types::PamGetentResponse>().await {
                    Ok(resp) => Ok(resp),
                    Err(err) => {
                        log::error!("Error decoding getent response: {}", err);
                        Err(libnss::interop::Response::Unavail)
                    }
                }
            } else {
                let text = res.text().await.unwrap_or_default();
                log::error!("getent request failed: {}", text);
                Err(libnss::interop::Response::Unavail)
            }
        }) {
            Ok(resp) => resp,
            Err(err) => {
                return err;
            }
        }
    };
}

pub struct RauthyNss;
