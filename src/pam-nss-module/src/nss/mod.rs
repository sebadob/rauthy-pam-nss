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
    ($path:expr) => {
        match $crate::RT.block_on(async {
            let (status, body) = match $crate::uds::get($path).await {
                Ok(r) => r,
                Err(err) => {
                    log::error!("Error connecting to UDS: {}", err);
                    return Err(libnss::interop::Response::TryAgain);
                }
            };

            if status.is_success() {
                match bincode::decode_from_slice::<$crate::api_types::PamGetentResponse, _>(
                    body.as_ref(),
                    bincode::config::standard(),
                ) {
                    Ok((resp, _)) => Ok(resp),
                    Err(err) => {
                        log::error!("Error decoding getent response: {}", err);
                        Err(libnss::interop::Response::Unavail)
                    }
                }
            } else {
                let text = String::from_utf8_lossy(body.as_ref());
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
