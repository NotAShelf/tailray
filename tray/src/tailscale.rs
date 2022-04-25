use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    process::Command,
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
    dns_name: String,
    #[serde(rename(deserialize = "HostName"))]
    hostname: String,
    #[serde(rename(deserialize = "TailscaleIPs"))]
    pub ips: Vec<String>,
}
#[derive(Serialize, Deserialize, Debug)]
struct User {
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
#[derive(Serialize, Deserialize, Debug)]

pub struct Status {
    // TODO: mutex
    #[serde(skip)]
    pub tailscale_up: bool,
    #[serde(rename(deserialize = "BackendState"))]
    backend_state: String,
    #[serde(rename(deserialize = "Self"))]
    pub this_machine: Machine,
    #[serde(rename(deserialize = "MagicDNSSuffix"))]
    magic_dnssuffix: String,
    #[serde(rename(deserialize = "Peer"))]
    pub peers: HashMap<String, Machine>,
    #[serde(rename(deserialize = "User"))]
    user: HashMap<String, User>,
}

pub fn get_raw_status() -> Result<Value, serde_json::Error> {
    serde_json::from_str::<Value>(&get_json())
}

pub fn get_status() -> Result<Status, serde_json::Error> {
    let mut st: Status = serde_json::from_str(get_json().as_str())?;
    st.tailscale_up = match st.backend_state.as_str() {
        "Running" => true,
        "Stopped" => false,
        _ => false,
    };
    dns_or_quote_hostname(&mut st.this_machine, &st.magic_dnssuffix);
    st.peers
        .values_mut()
        .for_each(|m: &mut Machine| dns_or_quote_hostname(m, &st.magic_dnssuffix));
    Ok(st)
}

fn get_json() -> String {
    let output = Command::new("tailscale")
        .arg("status")
        .arg("--json")
        .output()
        .expect("Fetch tailscale status fail.");
    String::from_utf8(output.stdout).expect("Unable to convert status output string.")
}

fn dns_or_quote_hostname(m: &mut Machine, dns_suffix: &str) {
    let base_name = trim_suffix(&m.dns_name, dns_suffix);
    m.display_name = match base_name {
        n if n.is_empty() => PeerKind::DNSName(sanitize_hostname(m.hostname.as_str())),
        base => PeerKind::HostName(base),
    }
}
fn has_suffix(name: &str, suffix: &str) -> bool {
    let name = name.trim_end_matches(".");
    let mut suffix = suffix.trim_end_matches(".");
    suffix = suffix.trim_start_matches(".");
    let name_base = name.trim_end_matches(suffix);
    name_base.len() < name.len() && name_base.ends_with(".")
}

fn trim_suffix(name: &str, suffix: &str) -> String {
    let mut new_name = name.clone();
    if has_suffix(name, &suffix) {
        new_name = new_name.trim_end_matches(".");
        let suffix = suffix.trim_start_matches(".").trim_end_matches(".");
        new_name = new_name.trim_end_matches(suffix);
    }
    new_name.trim_end_matches(".").to_string()
}

fn sanitize_hostname(hostname: &str) -> String {
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
        start = start + 1;
    }
    while start < end {
        if chars.nth(end - 1).unwrap().is_alphanumeric() {
            break;
        }
        end = end - 1;
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
