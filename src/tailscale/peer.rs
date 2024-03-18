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

pub fn copy_peer_ip(peer_ip: &str, notif_title: &str) {
    check_peer_ip(peer_ip);

    match copy_to_clipboard(peer_ip) {
        Ok(_) => {
            // Get IP from clipboard to verify
            match get_from_clipboard() {
                Ok(clip_ip) => {
                    // log success
                    info!("Copied IP address {} to the Clipboard", clip_ip);

                    // send a notification through dbus
                    let body = format!("Copied IP address {} to the Clipboard", clip_ip);
                    let _result = Notification::new()
                        .summary(notif_title)
                        .body(&body)
                        .icon("info")
                        .show();
                }

                Err(e) => {
                    let message = "Failed to get IP from clipboard";
                    error!("{}: {}", message, e);

                    let _result = Notification::new()
                        .summary(notif_title)
                        .body(&message)
                        .icon("error")
                        .show();
                }
            }
        }
        Err(e) => error!("Failed to copy IP to clipboard: {}", e),
    }
}
