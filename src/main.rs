mod clipboard;
mod pkexec;
mod svg;
mod tailscale;
mod tray;

use ksni::TrayService;
use std::thread::park;

fn main() {
    // initialize logger
    env_logger::init();

    // start the tray service
    TrayService::new(crate::tray::menu::SysTray::new()).spawn();

    // keep the main thread alive
    // NOTE: The documentation for park reads:
    // "A call to park does not guarantee that the thread will
    // remain parked forever, and callers should be prepared for this possibility."
    // hence the loop
    loop {
        park();
    }
}
