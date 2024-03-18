use crate::tailscale::dns;
use crate::tailscale::utils::{Machine, User};
use crate::tray::menu::Context;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, process::Command};
use which::which;

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

pub fn get_current_status() -> Context {
    let status: Status = get_status().unwrap();

    Context {
        ip: status.this_machine.ips[0].clone(),
        pkexec: which("pkexec").unwrap(),
        status,
    }
}

pub fn get_status_json() -> String {
    let output = Command::new("tailscale")
        .arg("status")
        .arg("--json")
        .output()
        .expect("Fetch tailscale status fail.");
    String::from_utf8(output.stdout).expect("Unable to convert status output string.")
}

pub fn get_status() -> Result<Status, serde_json::Error> {
    let mut st: Status = serde_json::from_str(get_status_json().as_str())?;
    let dnssuffix = st.magic_dnssuffix.to_owned();
    st.tailscale_up = match st.backend_state.as_str() {
        "Running" => true,
        "Stopped" => false,
        _ => false,
    };

    dns::dns_or_quote_hostname(&mut st.this_machine, &dnssuffix);
    st.peers
        .values_mut()
        .for_each(|m: &mut Machine| dns::dns_or_quote_hostname(m, &dnssuffix));
    Ok(st)
}
