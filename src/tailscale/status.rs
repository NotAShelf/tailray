use crate::tailscale::utils;
use crate::tailscale::utils::{Machine, User};
use crate::tray::menu::Context;
use log::{debug, error, warn};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, fmt, process::Command};

/// Possible errors that can occur when working with Tailscale status
#[derive(Debug)]
pub enum StatusError {
    /// Tailscale command failed
    CommandFailed(String),
    /// JSON parsing failed
    ParseFailed(String),
    /// Missing required data
    MissingData(String),
}

impl fmt::Display for StatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CommandFailed(msg) => write!(f, "Tailscale command failed: {msg}"),
            Self::ParseFailed(msg) => write!(f, "Failed to parse Tailscale status: {msg}"),
            Self::MissingData(msg) => write!(f, "Missing data in Tailscale status: {msg}"),
        }
    }
}

impl Error for StatusError {}

/// Represents the complete status of the Tailscale network
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Status {
    // Runtime status (not from JSON)
    #[serde(skip)]
    pub tailscale_up: bool,

    #[serde(rename = "BackendState", default)]
    backend_state: String,

    #[serde(rename = "Self", default)]
    pub this_machine: Machine,

    #[serde(rename = "MagicDNSSuffix", default)]
    magic_dnssuffix: String,

    #[serde(rename = "Peer", default)]
    pub peers: HashMap<String, Machine>,

    #[serde(rename = "User", default)]
    user: HashMap<String, User>,

    // Catch all other fields we might not know about
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

/// Gets the current Tailscale status
pub fn get() -> Result<Status, Box<dyn Error>> {
    // Get and parse the JSON status
    let status_json = get_json()?;

    let mut status: Status = match serde_json::from_str(&status_json) {
        Ok(status) => status,
        Err(e) => {
            error!("Failed to parse Tailscale status: {}", e);
            return Err(Box::new(StatusError::ParseFailed(format!(
                "{e}: {status_json}"
            ))));
        }
    };

    // Set tailscale_up based on backend state
    status.tailscale_up = matches!(status.backend_state.as_str(), "Running");
    debug!("Tailscale status: up={}", status.tailscale_up);

    // Set display names for all machines
    let dnssuffix = &status.magic_dnssuffix;
    utils::set_display_name(&mut status.this_machine, dnssuffix);

    for machine in status.peers.values_mut() {
        utils::set_display_name(machine, dnssuffix);
    }

    Ok(status)
}

/// Gets the current context for the system tray
pub fn get_current() -> Result<Context, Box<dyn Error>> {
    // Get status
    let status = get()?;

    // Check if we have at least one IP address
    if status.this_machine.ips.is_empty() {
        error!("This machine has no IP addresses");
        return Err(Box::new(StatusError::MissingData(
            "This machine has no IP addresses".into(),
        )));
    }

    Ok(Context {
        ip: status.this_machine.ips[0].clone(),
        status,
    })
}

/// Gets the raw JSON status from the tailscale command
pub fn get_json() -> Result<String, Box<dyn Error>> {
    let output = match Command::new("tailscale")
        .arg("status")
        .arg("--json")
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            error!("Failed to execute tailscale command: {}", e);
            return Err(Box::new(StatusError::CommandFailed(e.to_string())));
        }
    };

    if output.status.success() {
        let stdout = match String::from_utf8(output.stdout) {
            Ok(stdout) => stdout,
            Err(e) => {
                error!("Invalid UTF-8 in output: {}", e);
                return Err(Box::new(StatusError::ParseFailed(format!(
                    "Invalid UTF-8 in output: {e}"
                ))));
            }
        };

        if stdout.trim().is_empty() {
            warn!("Tailscale returned empty status");
            return Err(Box::new(StatusError::MissingData(
                "Tailscale returned empty status".into(),
            )));
        }

        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Tailscale status command failed: {}", stderr.trim());
        Err(Box::new(StatusError::CommandFailed(format!(
            "Tailscale status command failed: {}",
            stderr.trim()
        ))))
    }
}
