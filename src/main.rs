mod clipboard;
mod pkexec;
mod svg;
mod tailscale;
mod tray;

use ksni::TrayService;

fn main() {
    // initialize logger
    env_logger::init();

    // start the tray service
    TrayService::new(crate::tray::menu::SysTray::new()).spawn();

    // keep the main thread alive
    loop {
        std::thread::park();
    }
}
