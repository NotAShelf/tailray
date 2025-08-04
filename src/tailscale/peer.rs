use crate::clipboard::{copy, get};
use log::{error, info};
use notify_rust::Notification;
use std::error::Error;
use std::fmt;

/// Custom error type for peer operations
#[derive(Debug)]
pub enum PeerError {
    /// Error when IP address is empty or invalid
    InvalidIP(String),
    /// Error when clipboard operation fails
    ClipboardError(String),
    /// Error when notification fails
    NotificationError(String),
    /// Error when verification fails
    VerificationError(String),
}

impl fmt::Display for PeerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIP(msg) => write!(f, "Invalid IP address: {msg}"),
            Self::ClipboardError(msg) => write!(f, "Clipboard error: {msg}"),
            Self::NotificationError(msg) => write!(f, "Notification error: {msg}"),
            Self::VerificationError(msg) => write!(f, "Verification error: {msg}"),
        }
    }
}

impl Error for PeerError {}

/// Validates the peer IP address format
pub fn validate_peer_ip(peer_ip: &str) -> Result<(), PeerError> {
    if peer_ip.is_empty() {
        error!("IP address is empty");
        return Err(PeerError::InvalidIP("IP address is empty".into()));
    }

    // Basic IPv4/IPv6 validation
    if !peer_ip.contains('.') && !peer_ip.contains(':') {
        error!("Invalid IP address format: {}", peer_ip);
        return Err(PeerError::InvalidIP(format!("Invalid format: {peer_ip}")));
    }

    Ok(())
}

/// Copies peer IP to clipboard and notifies the user
///
/// # Arguments
/// * `peer_ip` - The IP address to copy
/// * `notif_body` - The notification message body
/// * `host` - Whether this is a host IP (true) or peer IP (false)
///
/// # Returns
/// * `Result<(), Box<dyn Error>>` - Success or error
pub fn copy_peer_ip(peer_ip: &str, notif_body: &str, host: bool) -> Result<(), Box<dyn Error>> {
    validate_peer_ip(peer_ip)?;

    copy(peer_ip).map_err(|e| {
        error!("Failed to copy IP to clipboard: {}", e);
        PeerError::ClipboardError(e.to_string())
    })?;

    let clip_ip = get().map_err(|e| {
        error!("Failed to verify clipboard contents: {}", e);
        PeerError::ClipboardError(e.to_string())
    })?;

    if clip_ip != peer_ip {
        error!(
            "Clipboard verification failed: expected '{}', got '{}'",
            peer_ip, clip_ip
        );
        return Err(Box::new(PeerError::VerificationError(
            "Clipboard content doesn't match the copied IP".into(),
        )));
    }

    let summary = format!("Copied {} IP address", if host { "host" } else { "peer" });
    info!("{} {} to clipboard", summary, clip_ip);

    Notification::new()
        .summary(&summary)
        .body(notif_body)
        .icon("tailscale")
        .timeout(3000)
        .show()
        .map_err(|e| {
            error!("Failed to show notification: {}", e);
            PeerError::NotificationError(e.to_string())
        })?;

    Ok(())
}
