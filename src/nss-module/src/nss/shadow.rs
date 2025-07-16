// use crate::api_types::{Getent, GetentRequest, PamGetentResponse};
// use crate::nss::RauthyNss;
// use crate::{init_syslog, load_config_nss, send_getent};
// use libnss::interop::Response;
// use libnss::shadow::{Shadow, ShadowHooks};
//
// impl ShadowHooks for RauthyNss {
//     fn get_all_entries() -> Response<Vec<Shadow>> {
//         init_syslog();
//
//         let config = load_config_nss!();
//         let url = config.url_getent();
//         let payload = GetentRequest {
//             host_id: config.host_id,
//             host_secret: config.host_secret,
//             getent: Getent::Users,
//         };
//
//         match send_getent!(url, payload) {
//             PamGetentResponse::Users(users) => {
//                 let mut res = Vec::with_capacity(users.len());
//
//                 for user in users {
//                     res.push(Shadow {
//                         name: user.name,
//                         passwd: "!".to_string(),
//                         last_change: -1,
//                         change_min_days: 0,
//                         change_max_days: 99999,
//                         change_warn_days: 7,
//                         change_inactive_days: -1,
//                         expire_date: -1,
//                         reserved: 0,
//                     });
//                 }
//
//                 Response::Success(res)
//             }
//             _ => unreachable!(),
//         }
//     }
//
//     fn get_entry_by_name(name: String) -> Response<Shadow> {
//         init_syslog();
//
//         let config = load_config_nss!();
//         let url = config.url_getent();
//         let payload = GetentRequest {
//             host_id: config.host_id,
//             host_secret: config.host_secret,
//             getent: Getent::Username(name),
//         };
//
//         match send_getent!(url, payload) {
//             PamGetentResponse::User(user) => Response::Success(Shadow {
//                 name: user.name,
//                 passwd: "!".to_string(),
//                 last_change: -1,
//                 change_min_days: 0,
//                 change_max_days: 99999,
//                 change_warn_days: 7,
//                 change_inactive_days: -1,
//                 expire_date: -1,
//                 reserved: 0,
//             }),
//             _ => unreachable!(),
//         }
//     }
// }
