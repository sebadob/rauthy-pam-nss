use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Serialize)]
pub enum Getent {
    Users,
    Username(String),
    UserId(u32),
    Groups,
    Groupname(String),
    GroupId(u32),
    Hosts,
    Hostname(String),
    HostIp(IpAddr),
}

#[derive(Debug, Serialize)]
pub struct GetentRequest<'a> {
    pub host_id: &'a str,
    pub host_secret: &'a str,
    pub getent: &'a Getent,
}

#[derive(Debug, Serialize)]
pub struct HostWhoamiRequest<'a> {
    pub host_secret: &'a str,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostResponse {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub addresses: Vec<IpAddr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostDetailsResponse {
    pub id: String,
    pub hostname: String,
    pub gid: u32,
    pub force_mfa: bool,
    pub notes: Option<String>,
    pub ips: Vec<IpAddr>,
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GroupType {
    Immutable,
    Host,
    User,
    Generic,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupResponse {
    pub id: u32,
    pub name: String,
    pub typ: GroupType,
    pub members: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: u32,
    pub name: String,
    pub gid: u32,
    pub email: String,
    pub shell: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GetentResponse {
    Users(Vec<UserResponse>),
    User(UserResponse),
    Groups(Vec<GroupResponse>),
    Group(GroupResponse),
    Hosts(Vec<HostResponse>),
    Host(HostResponse),
}
