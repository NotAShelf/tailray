use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fmt::{Display, Formatter},
};

#[derive(Debug)]
pub enum PeerKind {
    DNSName(String),
    HostName(String),
}

impl Default for PeerKind {
    fn default() -> Self {
        Self::HostName("default".to_owned())
    }
}

impl Display for PeerKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::DNSName(d) => write!(f, "{d}"),
            Self::HostName(h) => write!(f, "{h}"),
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
        let suffix = suffix.trim_start_matches('.').trim_end_matches('.');
        new_name = new_name.trim_end_matches('.');
        new_name = new_name.trim_end_matches(suffix);
    }
    new_name.trim_end_matches('.').to_string()
}

pub fn sanitize_hostname(hostname: &str) -> String {
    const MAX_LABEL_LENGTH: usize = 63;

    // Trim suffixes
    let hostname = hostname
        .trim_end_matches(".local")
        .trim_end_matches(".localdomain")
        .trim_end_matches(".lan");

    // Find the first/last alphanumeric characters
    let start = hostname.find(|c: char| c.is_alphanumeric()).unwrap_or(0);
    let end = hostname
        .rfind(|c: char| c.is_alphanumeric())
        .map_or(0, |e| e + 1);

    let separators: HashSet<char> = [' ', '.', '@', '_'].into();

    let mut sanitized: String = hostname[start..end]
        .chars()
        .enumerate()
        .map(|(index, char)| {
            let boundary = (index == 0) || (index == end - start - 1);
            if !boundary && separators.contains(&char) {
                '-'
            } else if char.is_alphanumeric() || char == '-' {
                char.to_ascii_lowercase()
            } else {
                char
            }
        })
        .collect();

    sanitized.truncate(MAX_LABEL_LENGTH);
    sanitized
}

pub fn set_display_name(m: &mut Machine, dns_suffix: &str) {
    let dns_name = trim_suffix(&m.dns_name, dns_suffix);

    if dns_name.is_empty() {
        m.display_name = PeerKind::DNSName(sanitize_hostname(&m.hostname));
    } else {
        m.display_name = PeerKind::HostName(dns_name);
    }
}
