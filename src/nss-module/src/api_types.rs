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
pub enum PamGroupType {
    Immutable,
    Host,
    User,
    Generic,
    Local,
}

#[derive(Debug, Decode)]
pub struct GroupResponse {
    pub id: u32,
    pub name: String,
    pub _typ: PamGroupType,
    pub members: Vec<String>,
}

#[derive(Debug, Decode)]
pub struct UserResponse {
    pub id: u32,
    pub name: String,
    pub gid: u32,
    pub email: String,
    pub shell: String,
    // The API does not return an `Option(_)`, but setting it increases compatibility and makes
    // it possible to migrate without any service downtime. Can be changed to `String` in the
    // future.
    pub home_dir: Option<String>,
}

impl UserResponse {
    pub fn home_dir(&self) -> String {
        self.home_dir
            .clone()
            .unwrap_or_else(|| format!("/home/{}", self.name))
    }
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
