use crate::api_types::PamGetentResponse;
use crate::nss::RauthyNss;
use crate::{init_syslog, send_getent};
use libnss::host::{AddressFamily, Addresses, Host, HostHooks};
use libnss::interop::Response;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

impl HostHooks for RauthyNss {
    fn get_all_entries() -> Response<Vec<Host>> {
        init_syslog();

        // let config = load_config_nss!();
        // let url = config.url_getent();
        // let payload = GetentRequest {
        //     host_id: config.host_id,
        //     host_secret: config.host_secret,
        //     getent: Getent::Hosts,
        // };

        match send_getent!("/getent/hosts") {
            PamGetentResponse::Hosts(hosts) => {
                let mut res = Vec::with_capacity(hosts.len());

                for host in hosts {
                    let (v4, v6) = split_addrs(host.addresses);
                    res.push(Host {
                        name: host.name,
                        aliases: host.aliases,
                        addresses: dyn_addr(v4, v6),
                    });
                }

                Response::Success(res)
            }
            _ => unreachable!(),
        }
    }

    fn get_host_by_name(name: &str, family: AddressFamily) -> Response<Host> {
        init_syslog();

        // let config = load_config_nss!();
        // let url = config.url_getent();
        // let payload = GetentRequest {
        //     host_id: config.host_id,
        //     host_secret: config.host_secret,
        //     getent: Getent::Hostname(name.to_string()),
        // };

        match send_getent!(&format!("/getent/hosts/name/{name}")) {
            PamGetentResponse::Host(host) => {
                let (v4, v6) = split_addrs(host.addresses);
                let addresses = match family {
                    AddressFamily::IPv4 => Addresses::V4(v4),
                    AddressFamily::IPv6 => Addresses::V6(v6),
                    AddressFamily::Unspecified => dyn_addr(v4, v6),
                };

                // TODO this request works and we actually get data, but `getent` does
                //  not display it? maybe config issue?

                Response::Success(Host {
                    name: host.name,
                    aliases: host.aliases,
                    addresses,
                })
            }
            _ => unreachable!(),
        }
    }

    fn get_host_by_addr(addr: IpAddr) -> Response<Host> {
        init_syslog();

        // let config = load_config_nss!();
        // let url = config.url_getent();
        // let payload = GetentRequest {
        //     host_id: config.host_id,
        //     host_secret: config.host_secret,
        //     getent: Getent::HostIp(addr),
        // };

        match send_getent!(&format!("/getent/hosts/ip/{addr}")) {
            PamGetentResponse::Host(host) => {
                let (v4, v6) = split_addrs(host.addresses);
                Response::Success(Host {
                    name: host.name,
                    aliases: host.aliases,
                    addresses: dyn_addr(v4, v6),
                })
            }
            _ => unreachable!(),
        }
    }
}

#[inline]
fn split_addrs(addrs: Vec<IpAddr>) -> (Vec<Ipv4Addr>, Vec<Ipv6Addr>) {
    let mut v4 = Vec::new();
    let mut v6 = Vec::new();

    for addr in addrs {
        match addr {
            IpAddr::V4(ip) => v4.push(ip),
            IpAddr::V6(ip) => v6.push(ip),
        }
    }

    (v4, v6)
}

#[inline]
fn dyn_addr(v4: Vec<Ipv4Addr>, v6: Vec<Ipv6Addr>) -> Addresses {
    if v4.is_empty() {
        Addresses::V6(v6)
    } else {
        Addresses::V4(v4)
    }
}
