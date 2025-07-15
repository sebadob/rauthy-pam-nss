use log::info;

pub async fn handler() -> &'static str {
    info!("new connection");

    "Hello, World!"
}

pub async fn get_getent() -> &'static str {
    info!("get_getent()");

    "GETENT"
}
