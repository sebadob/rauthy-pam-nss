use crate::api_types::GetentResponse;
use crate::nss::RauthyNss;
use crate::{init_syslog, send_getent};
use libc::gid_t;
use libnss::group::{Group, GroupHooks};
use libnss::interop::Response;

impl GroupHooks for RauthyNss {
    fn get_all_entries() -> Response<Vec<Group>> {
        init_syslog();

        match send_getent!("/getent/groups") {
            GetentResponse::Groups(groups) => Response::Success(
                groups
                    .into_iter()
                    .map(|g| Group {
                        name: g.name,
                        passwd: "x".to_string(),
                        gid: g.id,
                        members: g.members,
                    })
                    .collect::<Vec<_>>(),
            ),
            _ => unreachable!(),
        }
    }

    fn get_entry_by_gid(gid: gid_t) -> Response<Group> {
        // Rauthys ids start all at 100_000
        if gid < 100_000 {
            return Response::NotFound;
        }

        init_syslog();

        match send_getent!(&format!("/getent/groups/gid/{gid}")) {
            GetentResponse::Group(group) => Response::Success(Group {
                name: group.name,
                passwd: "x".to_string(),
                gid: group.id,
                members: group.members,
            }),
            _ => unreachable!(),
        }
    }

    fn get_entry_by_name(name: String) -> Response<Group> {
        init_syslog();

        match send_getent!(&format!("/getent/groups/name/{name}")) {
            GetentResponse::Group(group) => Response::Success(Group {
                name: group.name,
                passwd: "x".to_string(),
                gid: group.id,
                members: group.members,
            }),
            _ => unreachable!(),
        }
    }
}
