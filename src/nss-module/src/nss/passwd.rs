use crate::api_types::GetentResponse;
use crate::init_syslog;
use crate::{RauthyNss, send_getent};
use libnss::interop::Response;
use libnss::passwd::{Passwd, PasswdHooks};
use log::info;

impl PasswdHooks for RauthyNss {
    fn get_all_entries() -> Response<Vec<Passwd>> {
        init_syslog();

        info!("PasswdHooks get_all_entries");

        match send_getent!("/getent/users") {
            GetentResponse::Users(users) => {
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

        match send_getent!(&format!("/getent/users/uid/{uid}")) {
            GetentResponse::User(user) => {
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

        info!("PasswdHooks get_entry_by_name {name}");

        match send_getent!(&format!("/getent/users/name/{name}")) {
            GetentResponse::User(user) => {
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
