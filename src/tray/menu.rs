use crate::pkexec::{get_path_or_default, should_elevate_perms};
use crate::svg::renderer::Resvg;
use crate::tailscale::peer::copy_peer_ip;
use crate::tailscale::status::{Status, get_current};
use crate::tailscale::utils::PeerKind;

use ksni::{
    self, Icon, MenuItem, OfflineReason, ToolTip, Tray,
    menu::{StandardItem, SubMenu},
};

use log::{debug, error, info};
use notify_rust::Notification;
use std::{
    error::Error,
    fmt,
    process::{Command, Stdio},
};

/// Custom error type for `SystemTray` operations
#[derive(Debug)]
pub enum TrayError {
    Command(String),
    StatusUpdate(String),
    Notification(String),
}

impl fmt::Display for TrayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Command(msg) => write!(f, "Command execution failed: {msg}"),
            Self::StatusUpdate(msg) => write!(f, "Status update failed: {msg}"),
            Self::Notification(msg) => write!(f, "Notification failed: {msg}"),
        }
    }
}

impl Error for TrayError {}

/// Represents the context for the system tray
#[derive(Debug, Default)]
pub struct Context {
    pub ip: String,
    pub status: Status,
}

/// The main `SystemTray` implementation
#[derive(Debug)]
pub struct SysTray {
    pub ctx: Context,
}

impl SysTray {
    /// Returns whether Tailscale is enabled
    pub const fn enabled(&self) -> bool {
        self.ctx.status.tailscale_up
    }

    /// Updates the Tailscale status
    pub fn update_status(&mut self) -> Result<(), Box<dyn Error>> {
        match get_current() {
            Ok(ctx) => {
                self.ctx = ctx;
                Ok(())
            }
            Err(e) => {
                error!("Failed to update status: {e}");
                Err(Box::new(TrayError::StatusUpdate(e.to_string())))
            }
        }
    }

    /// Executes a Tailscale service command (up/down)
    pub fn do_service_link(&mut self, verb: &str) -> Result<(), Box<dyn Error>> {
        let elevate = should_elevate_perms();
        let (cmd, args) = if elevate {
            (get_path_or_default(), vec!["tailscale", verb])
        } else {
            ("tailscale".into(), vec![verb])
        };

        info!(
            "{} permissions for {}",
            if elevate {
                "Elevating"
            } else {
                "Running without elevation"
            },
            cmd.display()
        );

        let output = Command::new(cmd)
            .args(&args)
            .stdout(Stdio::piped())
            .spawn()
            .and_then(std::process::Child::wait_with_output)
            .map_err(|e| {
                error!("Failed to execute command: {e}");
                TrayError::Command(e.to_string())
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        info!("Link {}: [{}]{}", verb, output.status, stdout);

        let notify = |summary: &str, body: &str, icon: &str| {
            Notification::new()
                .summary(summary)
                .body(body)
                .icon(icon)
                .show()
                .map_err(|e| {
                    error!("Failed to show notification: {e}");
                    TrayError::Notification(e.to_string())
                })
        };

        if output.status.success() {
            notify(
                &format!("Connection {verb}"),
                &format!(
                    "Tailscale service {}",
                    if verb == "up" { "online" } else { "offline" }
                ),
                "info",
            )?;
            self.update_status()?;
        } else {
            error!("Failed to {verb} Tailscale: {stdout}");
            notify(&format!("Connection {verb} failed"), &stdout, "error")?;
        }

        Ok(())
    }
}

impl Tray for SysTray {
    fn icon_name(&self) -> String {
        if self.enabled() {
            "tailscale-online".into()
        } else {
            "tailscale-offline".into()
        }
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        Resvg::load_icon(self.enabled())
    }

    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }

    fn title(&self) -> String {
        "Tailray".into()
    }

    fn tool_tip(&self) -> ToolTip {
        let state = if self.enabled() {
            "Connected"
        } else {
            "Disconnected"
        };

        ToolTip {
            title: format!("Tailscale: {state}"),
            description: String::default(),
            icon_name: String::default(),
            icon_pixmap: Vec::default(),
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let my_ip = self.ctx.ip.clone();
        let device_name = self.ctx.status.this_machine.display_name.to_string();

        let message = format!("This device: {} ({})", device_name, self.ctx.ip);
        debug!("Creating menu with device {message}");

        // Prepare device submenus
        let (my_sub, serv_sub): (Vec<_>, Vec<_>) = self
            .ctx
            .status
            .peers
            .iter()
            .filter(|(_, peer)| !peer.ips.is_empty())
            .map(|(_, peer)| {
                let ip = peer.ips[0].clone();
                let name = &peer.display_name;
                let peer_title = format!("{name} ({ip})");
                let display_name = name.to_string();
                let menu = MenuItem::Standard(StandardItem {
                    label: format!("{display_name}\t({ip})"),
                    activate: Box::new(move |_: &mut Self| {
                        if let Err(e) = copy_peer_ip(&ip, &peer_title, false) {
                            error!("Failed to copy peer IP: {e}");
                        }
                    }),
                    ..Default::default()
                });
                match name {
                    PeerKind::DNSName(_) => (None, Some(menu)),
                    PeerKind::HostName(_) => (Some(menu), None),
                }
            })
            .fold(
                (Vec::new(), Vec::new()),
                |(mut my, mut serv), (my_item, serv_item)| {
                    if let Some(item) = my_item {
                        my.push(item);
                    }
                    if let Some(item) = serv_item {
                        serv.push(item);
                    }
                    (my, serv)
                },
            );

        vec![
            StandardItem {
                label: "Connect".into(),
                icon_name: "network-transmit-receive-symbolic".into(),
                enabled: !self.enabled(),
                visible: true,
                activate: Box::new(|this: &mut Self| {
                    if let Err(e) = this.do_service_link("up") {
                        error!("Failed to connect: {e}");
                    }
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Disconnect".into(),
                icon_name: "network-offline-symbolic".into(),
                enabled: self.enabled(),
                visible: true,
                activate: Box::new(|this: &mut Self| {
                    if let Err(e) = this.do_service_link("down") {
                        error!("Failed to disconnect: {e}");
                    }
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: message.clone(),
                icon_name: "computer-symbolic".into(),
                activate: Box::new(move |_| {
                    if let Err(e) = copy_peer_ip(&my_ip, &message, true) {
                        error!("Failed to copy IP for this device: {e}");
                    }
                }),
                ..Default::default()
            }
            .into(),
            SubMenu {
                label: "Network Devices".into(),
                icon_name: "network-wired-symbolic".into(),
                submenu: vec![
                    SubMenu {
                        label: "My Devices".into(),
                        submenu: my_sub,
                        ..Default::default()
                    }
                    .into(),
                    SubMenu {
                        label: "Tailscale Services".into(),
                        submenu: serv_sub,
                        ..Default::default()
                    }
                    .into(),
                ],
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Admin Console".into(),
                icon_name: "applications-system-symbolic".into(),
                activate: Box::new(|_| {
                    let admin_url = std::env::var("TAILRAY_ADMIN_URL").unwrap_or_else(|_| {
                        "https://login.tailscale.com/admin/machines".to_string()
                    });

                    if let Err(e) = open::that(&admin_url) {
                        error!("Failed to open admin console: {e}");
                    }
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Exit Tailray".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|_| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        ]
    }

    fn watcher_online(&self) {
        info!("Watcher online.");
    }

    fn watcher_offline(&self, reason: OfflineReason) -> bool {
        info!("Watcher offline, signaling for reconnection: {reason:?}");

        // Signal the watchdog to respawn the tray
        crate::tray::utils::signal_respawn_needed();

        // Return false to allow the current instance to be cleaned up
        // The watchdog will spawn a new one
        false
    }
}
