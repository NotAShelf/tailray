use crate::clipboard::{copy_to_clipboard, get_from_clipboard};
use log::{error, info};
use notify_rust::Notification;

pub fn check_peer_ip(peer_ip: &str) {
    if peer_ip.is_empty() {
        error!("No peer IP.")
    } else {
        info!("Peer IP: {}", peer_ip);
    }
}

pub fn copy_peer_ip(
    peer_ip: &str,
    notif_body: &str,
    host: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    check_peer_ip(peer_ip);

    copy_to_clipboard(peer_ip)?;

    // Get IP from clipboard to verify
    let clip_ip = get_from_clipboard()?;

    // Create summary for host/peer
    let summary = format!("Copied {} IP address", if host { "host" } else { "peer" });

    // log success
    info!("{} {} to the clipboard", summary, clip_ip);

    // send a notification through dbus
    Notification::new()
        .summary(&summary)
        .body(&notif_body)
        .icon("info")
        .show()?;

    Ok(())
}
