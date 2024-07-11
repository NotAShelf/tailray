use crate::tailscale::dns;
use crate::tailscale::utils::{Machine, User};
use crate::tray::menu::Context;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io, process::Command};
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

pub fn get_status() -> Result<Status, Box<dyn std::error::Error>> {
    let status_json = get_status_json()?;
    let mut status: Status = serde_json::from_str(&status_json)?;
    let dnssuffix = status.magic_dnssuffix.to_owned();
    status.tailscale_up = matches!(status.backend_state.as_str(), "Running");

    dns::dns_or_quote_hostname(&mut status.this_machine, &dnssuffix);
    status
        .peers
        .values_mut()
        .for_each(|m| dns::dns_or_quote_hostname(m, &dnssuffix));

    Ok(status)
}

pub fn get_current_status() -> Result<Context, Box<dyn std::error::Error>> {
    let status = get_status()?;
    let pkexec = which("pkexec")?;

    Ok(Context {
        ip: status.this_machine.ips[0].clone(),
        pkexec,
        status,
    })
}

pub fn get_status_json() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("tailscale")
        .arg("status")
        .arg("--json")
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)?;
        Ok(stdout)
    } else {
        Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            "Failed to fetch tailscale status.",
        )))
    }
}
