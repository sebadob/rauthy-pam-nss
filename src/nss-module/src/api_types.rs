use bincode::Decode;
use std::net::IpAddr;

#[allow(dead_code)]
#[derive(Debug, Decode)]
pub struct HostResponse {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub addresses: Vec<IpAddr>,
}

#[derive(Debug, Decode)]
pub struct GroupResponse {
    pub id: u32,
    pub name: String,
    // Vec<{username}>
    pub members: Vec<String>,
}

#[derive(Debug, Decode)]
pub struct UserResponse {
    pub id: u32,
    pub name: String,
    pub gid: u32,
    pub email: String,
    pub shell: String,
}

#[derive(Debug, Decode)]
pub enum GetentResponse {
    Users(Vec<UserResponse>),
    User(UserResponse),
    Groups(Vec<GroupResponse>),
    Group(GroupResponse),
    Hosts(Vec<HostResponse>),
    Host(HostResponse),
}
