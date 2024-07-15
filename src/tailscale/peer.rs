use crate::clipboard::{copy, get};
use notify_rust::Notification;

pub fn check_peer_ip(peer_ip: &str) {
    if peer_ip.is_empty() {
        log::error!("No peer IP.");
    } else {
        log::info!("Peer IP: {peer_ip}");
    }
}

pub fn copy_peer_ip(
    peer_ip: &str,
    notif_body: &str,
    host: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    check_peer_ip(peer_ip);

    copy(peer_ip)?;

    // Get IP from clipboard to verify
    let clip_ip = get()?;

    // Create summary for host/peer
    let summary = format!("Copied {} IP address", if host { "host" } else { "peer" });

    // log success
    log::info!("{summary} {clip_ip} to the clipboard");

    // send a notification through dbus
    Notification::new()
        .summary(&summary)
        .body(notif_body)
        .icon("tailscale")
        .show()?;

    Ok(())
}
