use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use ksni::{
    self, menu::{CheckmarkItem, StandardItem, SubMenu},
    Category, Icon, MenuItem, Status,
    Tray, TextDirection, ToolTip, TrayService,
};
use log::info;
use notify_rust::Notification;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};
use which::which;

#[derive(Debug)]
struct Context {
    // TODO: rwlock
    ip: String,
    pkexec: PathBuf,
}

#[derive(Debug)]
struct MyTray {
    ctx: Context,
    checked: bool,
}

impl MyTray {
    fn do_service_link(verb: &str) {
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
        self.ctx.pkexec.is_file()
    }
    fn copy_ip(&self) {
        if self.ctx.ip.is_empty() {
            return;
        }
        let mut cctx: ClipboardContext = ClipboardProvider::new().unwrap();
        cctx.set_contents(self.ctx.ip.to_owned()).unwrap();
        info!("copy ip: {:?}", cctx.get_contents());

        let _result = Notification::new()
            .summary("This device")
            .body(format!(
                "Copy the IP address {:?} to the Clipboard",
                cctx.get_contents()
            ).as_str())
            .icon("info")
            .show();
    }
}

impl Tray for MyTray {
    fn title(&self) -> String {
        if self.checked { "CHECKED!" } else { "MyTray" }.into()
    }

    fn icon_name(&self) -> String {
        "network".into()
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        // use ksnay::menu::*;
        // let pkexec_bin = find_exec("pkexec");
        let pkexec_exist: bool = self.pkexec_found();
        // let permissions = pkexec_bin.metadata()?.permissions();
        // let is_executable = permissions.mode() & 0o111 != 0;
        vec![
            StandardItem {
                label: "Connect".into(),
                icon_name: "network-transmit-receive".into(),
                enabled: true,
                visible: pkexec_exist,
                activate: Box::new(|_| MyTray::do_service_link("up")),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Disconnect".into(),
                icon_name: "network-offline".into(),
                enabled: false,
                visible: pkexec_exist,
                activate: Box::new(|_| MyTray::do_service_link("down")),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "This device:".into(),
                activate: Box::new(|t: &mut MyTray| t.copy_ip()),
                ..Default::default()
            }
            .into(),
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

    fn watcher_online(&self) {

    }

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

    let tray = MyTray { ctx, checked: false};
    let service = TrayService::new(tray);

    let handle = service.handle();
    service.spawn();

    handle.update(|tray: &mut MyTray| {
        tray.checked = true;
        // TODO: update ip
    });
    loop {
        std::thread::park();
    }
}
