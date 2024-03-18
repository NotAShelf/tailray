use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

#[derive(Debug)]
pub enum PeerKind {
    DNSName(String),
    HostName(String),
}

impl Default for PeerKind {
    fn default() -> Self {
        PeerKind::HostName("default".to_owned())
    }
}

impl Display for PeerKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            PeerKind::DNSName(d) => write!(f, "{d}"),
            PeerKind::HostName(h) => write!(f, "{h}"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Machine {
    #[serde(skip)]
    pub display_name: PeerKind,
    #[serde(rename(deserialize = "DNSName"))]
    pub dns_name: String,
    #[serde(rename(deserialize = "HostName"))]
    pub hostname: String,
    #[serde(rename(deserialize = "TailscaleIPs"))]
    pub ips: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    #[serde(rename(deserialize = "ID"))]
    id: u64,
    #[serde(rename(deserialize = "LoginName"))]
    login_name: String,
    #[serde(rename(deserialize = "DisplayName"))]
    display_name: String,
    #[serde(rename(deserialize = "ProfilePicURL"))]
    profile_pic_url: String,
    #[serde(rename(deserialize = "Roles"))]
    roles: Vec<String>,
}

pub fn has_suffix(name: &str, suffix: &str) -> bool {
    let name = name.trim_end_matches('.');
    let mut suffix = suffix.trim_end_matches('.');
    suffix = suffix.trim_start_matches('.');
    let name_base = name.trim_end_matches(suffix);
    name_base.len() < name.len() && name_base.ends_with('.')
}

pub fn trim_suffix(name: &str, suffix: &str) -> String {
    let mut new_name = name;
    if has_suffix(name, suffix) {
        new_name = new_name.trim_end_matches('.');
        let suffix = suffix.trim_start_matches('.').trim_end_matches('.');
        new_name = new_name.trim_end_matches(suffix);
    }
    new_name.trim_end_matches('.').to_string()
}

pub fn sanitize_hostname(hostname: &str) -> String {
    const MAX_LABEL_LEN: usize = 63;
    let mut sb = "".to_string();
    let hostname = hostname
        .trim_end_matches(".local")
        .trim_end_matches(".localdomain")
        .trim_end_matches(".lan");
    let mut start = 0;
    let mut end = hostname.len();
    if end > MAX_LABEL_LEN {
        end = MAX_LABEL_LEN;
    }
    let mut chars = hostname.chars();
    while start < end {
        if chars.nth(start).unwrap().is_alphanumeric() {
            break;
        }
        start += 1;
    }
    while start < end {
        if chars.nth(end - 1).unwrap().is_alphanumeric() {
            break;
        }
        end -= 1;
    }
    let seperators: HashMap<char, bool> =
        HashMap::from([(' ', true), ('.', true), ('@', true), ('_', true)]);

    let mut chars = hostname.chars();
    for i in start..end - 1 {
        let boundary = (i == start) || (i == end - 1);
        let chari = chars.nth(i).unwrap();
        if !boundary && seperators[&chari] {
            sb.push('-');
        } else if chari.is_alphanumeric() || chari == '-' {
            sb.push(chari.to_ascii_lowercase())
        }
    }
    sb
}
