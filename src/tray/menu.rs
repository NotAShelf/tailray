use crate::pkexec::{get_pkexec_path, should_elevate_perms};
use crate::svg::renderer::ResvgRenderer;
use crate::tailscale::peer::copy_peer_ip;
use crate::tailscale::status::{get_current_status, Status};
use crate::tailscale::utils::PeerKind;

use ksni::{
    self,
    menu::{StandardItem, SubMenu},
    Icon, MenuItem, ToolTip, Tray,
};

use log::info;
use notify_rust::Notification;
use std::{
    error::Error,
    path::PathBuf,
    process::{Command, Stdio},
};

#[derive(Debug)]
pub struct Context {
    pub ip: String,
    pub pkexec: PathBuf,
    pub status: Status,
}

#[derive(Debug)]
pub struct SysTray {
    pub ctx: Context,
}

impl SysTray {
    fn enabled(&self) -> bool {
        self.ctx.status.tailscale_up
    }

    fn update_status(&mut self) -> Result<(), Box<dyn Error>> {
        self.ctx = get_current_status()?;
        Ok(())
    }

    fn do_service_link(&mut self, verb: &str) -> Result<(), Box<dyn Error>> {
        let pkexec_path = get_pkexec_path();
        let elevate = should_elevate_perms();
        let command = if elevate {
            info!("Elevating permissions for pkexec.");
            Command::new(pkexec_path)
                .arg("tailscale")
                .arg(verb)
                .stdout(Stdio::piped())
                .spawn()
        } else {
            Command::new("tailscale")
                .arg(verb)
                .stdout(Stdio::piped())
                .spawn()
        };

        let command = match command {
            Ok(cmd) => cmd,
            Err(err) => return Err(Box::new(err)),
        };

        let output = command.wait_with_output()?;

        info!(
            "Link {}: [{}]{}",
            &verb,
            output.status,
            String::from_utf8_lossy(&output.stdout)
        );

        if output.status.success() {
            let verb_result = if verb.eq("up") {
                "connected"
            } else {
                "disconnected"
            };

            Notification::new()
                .summary(format!("Connection {}", verb).as_str())
                .body(format!("Tailscale service {verb_result}").as_str())
                .icon("info")
                .show()?;

            self.update_status()?;
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
        ResvgRenderer::load_icon(self.enabled())
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

        // FIXME: the icon is still not showing up
        // ags returns:
        // "Error: can't assign "tailscale-online" as icon, it is not a file nor a named icon"
        ToolTip {
            title: format!("Tailscale: {}", state),
            description: Default::default(),
            icon_name: Default::default(),
            icon_pixmap: Default::default(),
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let my_ip = self.ctx.ip.clone();

        let mut my_sub = Vec::new();
        let mut serv_sub = Vec::new();
        for (_, peer) in self.ctx.status.peers.iter() {
            let ip = peer.ips[0].clone();
            let name = &peer.display_name;
            let title = name.to_string();
            let sub = match name {
                PeerKind::DNSName(_) => &mut serv_sub,
                PeerKind::HostName(_) => &mut my_sub,
            };

            let peer_ip = ip.to_owned();
            let peer_title = title.to_owned();
            let menu = MenuItem::Standard(StandardItem {
                label: format!("{}\t({})", title, ip),
                activate: Box::new(move |_: &mut Self| {
                    if let Err(e) = copy_peer_ip(&peer_ip, &peer_title) {
                        eprintln!("failed to copy peer ip: {}", e);
                    }
                }),
                ..Default::default()
            });
            sub.push(menu);
        }
        vec![
            StandardItem {
                label: "Connect".into(),
                icon_name: "network-transmit-receive".into(),
                enabled: !self.enabled(),
                visible: true,
                activate: Box::new(|this: &mut Self| {
                    if let Err(e) = this.do_service_link("up") {
                        eprintln!("failed to connect: {}", e);
                    }
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Disconnect".into(),
                icon_name: "network-offline".into(),
                enabled: self.enabled(),
                visible: true,
                activate: Box::new(|this: &mut Self| {
                    if let Err(e) = this.do_service_link("down") {
                        eprintln!("failed to disconnect: {}", e);
                    }
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: format!(
                    "This device: {} ({})",
                    self.ctx.status.this_machine.display_name, self.ctx.ip
                ),
                activate: Box::new(move |_| {
                    if let Err(e) = copy_peer_ip(&my_ip, "Peer IP copied to clipboard!") {
                        eprintln!("failed to copy ip for this device: {}", e);
                    }
                }),
                ..Default::default()
            }
            .into(),
            SubMenu {
                label: "Network Devices".into(),
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
                label: "Admin Console…".into(),
                activate: Box::new(|_| {
                    let admin_url = std::env::var("TAILRAY_ADMIN_URL").unwrap_or_else(|_| {
                        "https://login.tailscale.com/admin/machines".to_string()
                    });
                    if let Err(e) = open::that(admin_url.as_str()) {
                        eprintln!("failed to open admin console: {}", e);
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
        info!("watcher online.");
    }

    fn watcher_offline(&self) -> bool {
        info!("watcher offline, shutting down the system tray.");
        false
    }
}
