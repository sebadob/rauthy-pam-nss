use crate::api_types::{Getent, PamGetentRequest, PamGetentResponse};
use crate::nss::RauthyNss;
use crate::{init_syslog, load_config_nss, send_getent};
use libnss::interop::Response;
use libnss::passwd::{Passwd, PasswdHooks};

impl PasswdHooks for RauthyNss {
    fn get_all_entries() -> Response<Vec<Passwd>> {
        init_syslog();

        let config = load_config_nss!();
        let url = config.url_getent();
        let payload = PamGetentRequest {
            host_id: config.host_id,
            host_secret: config.host_secret,
            getent: Getent::Users,
        };

        match send_getent!(url, payload) {
            PamGetentResponse::Users(users) => {
                let mut res = Vec::with_capacity(users.len());

                for user in users {
                    let dir = format!("/home/{}", user.name);
                    res.push(Passwd {
                        name: user.name,
                        passwd: "x".to_string(),
                        uid: user.id,
                        gid: user.gid,
                        gecos: user.email,
                        dir,
                        shell: user.shell,
                    });
                }

                Response::Success(res)
            }
            _ => unreachable!(),
        }
    }

    fn get_entry_by_uid(uid: libc::uid_t) -> Response<Passwd> {
        // Rauthys ids start all at 100_000
        if uid < 100_000 {
            return Response::NotFound;
        }

        init_syslog();

        let config = load_config_nss!();
        let url = config.url_getent();
        let payload = PamGetentRequest {
            host_id: config.host_id,
            host_secret: config.host_secret,
            getent: Getent::UserId(uid),
        };

        match send_getent!(url, payload) {
            PamGetentResponse::User(user) => {
                let dir = format!("/home/{}", user.name);
                Response::Success(Passwd {
                    name: user.name,
                    passwd: "x".to_string(),
                    uid: user.id,
                    gid: user.gid,
                    gecos: user.email,
                    dir,
                    shell: user.shell,
                })
            }
            _ => unreachable!(),
        }
    }

    fn get_entry_by_name(name: String) -> Response<Passwd> {
        init_syslog();

        let config = load_config_nss!();
        let url = config.url_getent();
        let payload = PamGetentRequest {
            host_id: config.host_id,
            host_secret: config.host_secret,
            getent: Getent::Username(name),
        };

        match send_getent!(url, payload) {
            PamGetentResponse::User(user) => {
                let dir = format!("/home/{}", user.name);
                Response::Success(Passwd {
                    name: user.name,
                    passwd: "x".to_string(),
                    uid: user.id,
                    gid: user.gid,
                    gecos: user.email,
                    dir,
                    shell: user.shell,
                })
            }
            _ => unreachable!(),
        }
    }
}
