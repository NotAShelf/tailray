mod svg;
mod tailscale;

use clipboard::{ClipboardContext, ClipboardProvider};
use ksni::{
    self,
    menu::{StandardItem, SubMenu},
    Icon, MenuItem, ToolTip, Tray, TrayService,
};
use log::{debug, error, info};
use notify_rust::Notification;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};
use tailscale::PeerKind;
use which::which;

#[derive(Debug)]
struct Context {
    // TODO: arc mutex
    ip: String,
    pkexec: PathBuf,
    status: tailscale::Status,
}

struct MyTray {
    ctx: Context,
    enabled: bool,
}
impl MyTray {
    pub fn new() -> Self {
        let status: tailscale::Status = tailscale::get_status().unwrap();
        let ctx = Context {
            ip: status.this_machine.ips[0].clone(),
            pkexec: which("pkexec").unwrap(),
            status,
        };
        assert_eq!(ctx.pkexec, PathBuf::from("/usr/bin/pkexec"));

        MyTray {
            enabled: ctx.status.tailscale_up,
            ctx,
        }
    }

    fn do_service_link(&self, verb: &str) {
        // consider using https://stackoverflow.com/a/66292796/554150
        // or async? https://rust-lang.github.io/async-book/03_async_await/01_chapter.html
        let child = Command::new("/usr/bin/pkexec")
            .arg("tailscale")
            .arg(verb.clone())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to execute process");

        let output = child.wait_with_output().expect("Failed to read stdout");
        info!("{}", String::from_utf8_lossy(&output.stdout));
        // expect_notify(title="Tailscale", message=output);

        let _result = Notification::new()
            .summary(format!("Connection {}", verb).as_str())
            .body("Tailscale service connected")
            .icon("info")
            .show();
    }
    fn pkexec_found(&self) -> bool {
        // let permissions = pkexec_bin.metadata()?.permissions();
        // let is_executable = permissions.mode() & 0o111 != 0;
        self.ctx.pkexec.is_file()
    }

    fn copy_peer_ip(peer_ip: &str, notif_title: &str) {
        if peer_ip.is_empty() {
            debug!("no ip");
            return;
        }
        let mut cctx: ClipboardContext =
            ClipboardProvider::new().expect("Clipboard unable to access.");
        match cctx.set_contents(peer_ip.to_owned()) {
            Ok(()) => {
                let clip_ip = cctx.get_contents().unwrap();
                info!("copy ip: {:?}", clip_ip);
                let _result = Notification::new()
                    .summary(notif_title)
                    .body(format!("Copy the IP address {clip_ip} to the Clipboard").as_str())
                    .icon("info")
                    .show();
            }
            Err(_) => error!("Unable to copy ip to clipboard."),
        }
    }
}

impl Tray for MyTray {
    fn title(&self) -> String {
        "Tailscale Tray".into()
    }
    fn tool_tip(&self) -> ToolTip {
        let state = if self.enabled {
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
        if self.enabled {
            "tailscale-online".into()
        } else {
            "tailscale-offline".into()
        }
    }
    fn icon_pixmap(&self) -> Vec<Icon> {
        // TODO: fix setting icon
        svg::load_icon(self.enabled)
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let pkexec_exist: bool = self.pkexec_found();
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
                activate: Box::new(move |_: &mut Self| Self::copy_peer_ip(&peer_ip, &peer_title)),
                ..Default::default()
            });
            sub.push(menu);
        }
        vec![
            StandardItem {
                label: "Connect".into(),
                icon_name: "network-transmit-receive".into(),
                enabled: !self.enabled,
                visible: pkexec_exist,
                activate: Box::new(|this: &mut Self| this.do_service_link("up")),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Disconnect".into(),
                icon_name: "network-offline".into(),
                enabled: self.enabled,
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
                )
                .into(),
                activate: Box::new(move |_| Self::copy_peer_ip(&my_ip, "This device")),
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
}

fn main() {
    env_logger::init();
    TrayService::new(MyTray::new()).spawn();
    loop {
        std::thread::park();
    }
}
