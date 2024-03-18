use crate::pkexec::{get_pkexec_path, pkexec_found};
use crate::svg::utils::ResvgRenderer;
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
    pub fn new() -> Self {
        SysTray {
            ctx: get_current_status(),
        }
    }

    fn enabled(&self) -> bool {
        self.ctx.status.tailscale_up
    }

    fn update_status(&mut self) {
        self.ctx = get_current_status();
    }

    fn do_service_link(&mut self, verb: &str) {
        let pkexec_path = get_pkexec_path();

        // TODO: consider using https://stackoverflow.com/a/66292796/554150
        // or async? https://rust-lang.github.io/async-book/03_async_await/01_chapter.html
        let child = Command::new(pkexec_path)
            .arg("tailscale")
            .arg(verb)
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to execute process");

        let output = child.wait_with_output().expect("Failed to read stdout");
        info!(
            "link {}: [{}]{}",
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
            let _result = Notification::new()
                .summary(format!("Connection {}", verb).as_str())
                .body(format!("Tailscale service {verb_result}").as_str())
                .icon("info")
                .show();
            self.update_status();
        }
    }
}

impl Tray for SysTray {
    fn title(&self) -> String {
        "Tailscale Tray".into()
    }

    fn tool_tip(&self) -> ToolTip {
        let state = if self.enabled() {
            "Connected"
        } else {
            "Disconnected"
        };

        ToolTip {
            icon_name: Default::default(),
            icon_pixmap: Default::default(),
            title: format!("Tailscale: {}", state),
            description: Default::default(),
        }
    }

    fn icon_name(&self) -> String {
        if self.enabled() {
            "tailscale-online".into()
        } else {
            "tailscale-offline".into()
        }
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        // TODO: fix setting icon
        ResvgRenderer::load_icon(self.enabled())
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let pkexec_path = get_pkexec_path();
        let pkexec_exist: bool = pkexec_found(&pkexec_path);
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
                activate: Box::new(move |_: &mut Self| copy_peer_ip(&peer_ip, &peer_title)),
                ..Default::default()
            });
            sub.push(menu);
        }
        vec![
            StandardItem {
                label: "Connect".into(),
                icon_name: "network-transmit-receive".into(),
                enabled: !self.enabled(),
                visible: pkexec_exist,
                activate: Box::new(|this: &mut Self| this.do_service_link("up")),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Disconnect".into(),
                icon_name: "network-offline".into(),
                enabled: self.enabled(),
                visible: pkexec_exist,
                activate: Box::new(|this: &mut Self| this.do_service_link("down")),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: format!(
                    "This device: {} ({})",
                    self.ctx.status.this_machine.display_name, self.ctx.ip
                ),
                activate: Box::new(move |_| copy_peer_ip(&my_ip, "This device")),
                ..Default::default()
            }
            .into(),
            SubMenu {
                label: "Network Devices:".into(),
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
                    open::that("https://login.tailscale.com/admin/machines").unwrap()
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Exit".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|_| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        ]
    }

    fn watcher_online(&self) {
        info!("Wathcer online.");
    }

    fn watcher_offine(&self) -> bool {
        info!("Wathcer offline, shutdown the tray.");
        false
    }

    fn id(&self) -> String {
        "mytray".to_string()
    }
}
