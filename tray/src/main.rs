mod svg;
mod tailscale;

use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use ksni::{
    self, Icon,
    menu::{CheckmarkItem, StandardItem, SubMenu},
    MenuItem, ToolTip, Tray, TrayService,
};
use log::{debug, info};
use notify_rust::Notification;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};
use which::which;

#[derive(Debug)]
struct Context {
    // TODO: arc mutex
    ip: String,
    pkexec: PathBuf,
}

struct MyTray {
    ctx: Context,
    enabled: bool,
    checked: bool,
}

impl MyTray {
    pub fn new(ctx: Context) -> Self {
        MyTray {
            ctx,
            enabled: false,
            checked: false,
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
    fn copy_ip(&self) {
        if self.ctx.ip.is_empty() {
            debug!("no ip");
            return;
        }
        let mut cctx: ClipboardContext = ClipboardProvider::new().unwrap();
        cctx.set_contents(self.ctx.ip.to_owned()).unwrap();
        info!("copy ip: {:?}", cctx.get_contents());

        let _result = Notification::new()
            .summary("This device")
            .body(
                format!(
                    "Copy the IP address {:?} to the Clipboard",
                    cctx.get_contents()
                )
                .as_str(),
            )
            .icon("info")
            .show();
    }
    pub fn init(&mut self) -> () {
        info!("init");
        let status: tailscale::Status = tailscale::get_status().unwrap();
        self.ctx.ip = status.this_machine.ips[0].clone();
    }
    fn _menu(&self) -> Vec<StandardItem<MyTray>> {
        let pkexec_exist: bool = self.pkexec_found();

        let m_connect = StandardItem {
            label: "Connect".into(),
            icon_name: "network-transmit-receive".into(),
            enabled: self.enabled,
            visible: pkexec_exist,
            activate: Box::new(|this: &mut Self| this.do_service_link("up")),
            ..Default::default()
        };
        let m_disconnect = StandardItem {
            label: "Disconnect".into(),
            icon_name: "network-offline".into(),
            enabled: !self.enabled,
            visible: pkexec_exist,
            activate: Box::new(|this: &mut Self| this.do_service_link("down")),
            ..Default::default()
        };
        let m_this = StandardItem {
            label: format!("This device: {}", self.ctx.ip).into(),
            activate: Box::new(|this: &mut Self| this.copy_ip()),
            ..Default::default()
        };
        vec![m_connect, m_disconnect, m_this]
    }
}

impl Tray for MyTray {
    fn title(&self) -> String {
        if self.checked { "CHECKED!" } else { "MyTray" }.into()
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
        svg::load()
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let mut m = self._menu();
        let m_this = m.pop().unwrap();
        let m_disconnect = m.pop().unwrap();
        let m_connect = m.pop().unwrap();
        vec![
            m_connect.into(),
            m_disconnect.into(),
            MenuItem::Separator,
            m_this.into(),
            SubMenu {
                label: "Network Devices:".into(),
                submenu: vec![
                    StandardItem {
                        label: "My Devices".into(),
                        ..Default::default()
                    }
                    .into(),
                    StandardItem {
                        label: "Tailscale Services".into(),
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
            CheckmarkItem {
                label: "Checkable".into(),
                checked: self.checked,
                activate: Box::new(|this: &mut Self| this.checked = !this.checked),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Exit".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|_| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        ]
    }

    fn watcher_online(&self) {}

    fn watcher_offine(&self) -> bool {
        info!("Wathcer offline, shutdown the tray.");
        false
    }
}

fn main() {
    env_logger::init();

    let pkexec = which("pkexec").unwrap();
    assert_eq!(pkexec, PathBuf::from("/usr/bin/pkexec"));
    let ctx = Context {
        pkexec,
        ip: "".to_owned(),
    };

    let mut tray = MyTray::new(ctx);
    tray.init();

    // TODO: need a map of menu items of node with ip

    let service = TrayService::new(tray);
    let handle = service.handle();
    service.spawn();
    handle.update(|tray: &mut MyTray| {
        tray.checked = true;
    });

    loop {
        std::thread::park();
    }
}
