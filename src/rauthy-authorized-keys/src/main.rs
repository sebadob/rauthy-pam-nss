use crate::config::Config;
use reqwest::header::ACCEPT;
use std::env;

mod config;
mod http_client;

pub fn main() {
    let config = Config::read().expect("Cannot load config");

    let username = {
        let mut args: Vec<String> = env::args().collect();
        if args.len() == 1 {
            eprintln!("Missing username as first argument");
            return;
        }
        let username = args.swap_remove(1);
        if username.is_empty() {
            eprintln!("username as first argument must not be empty");
            return;
        }
        username
    };

    if let Err(err) = run(config, &username) {
        eprintln!("Error running AuthorizedKeys lookup: {:?}", err);
    }
}

fn run(config: Config, username: &str) -> anyhow::Result<()> {
    let client = http_client::build(&config);

    let url = format!(
        "{}/auth/v1/pam/users/authorized_keys/{}",
        config.rauthy_url, username
    );
    let res = client
        .get(url)
        .header(ACCEPT, "text/plain")
        .basic_auth(config.host_id, Some(config.host_secret))
        .send()?;

    let status = res.status();
    if status.is_success() {
        // the body will already be a correctly formatted output
        let bytes = res.bytes()?;
        let keys = String::from_utf8_lossy(bytes.as_ref());
        println!("{keys}");
    } else {
        let bytes = res.bytes()?;
        let err = String::from_utf8_lossy(bytes.as_ref());
        eprintln!("/authorized_keys error: HTTPS {status} - {err}")
    }

    Ok(())
}
