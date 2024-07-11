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

pub fn copy_peer_ip(peer_ip: &str, notif_title: &str) -> Result<(), Box<dyn std::error::Error>> {
    check_peer_ip(peer_ip);

    copy_to_clipboard(peer_ip)?;

    // Get IP from clipboard to verify
    let clip_ip = get_from_clipboard()?;

    // log success
    info!("Copied IP address {} to the Clipboard", clip_ip);

    // send a notification through dbus
    let body = format!("Copied IP address {} to the Clipboard", clip_ip);
    Notification::new()
        .summary(notif_title)
        .body(&body)
        .icon("info")
        .show()?;

    Ok(())
}
