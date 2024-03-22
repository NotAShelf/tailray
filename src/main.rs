mod clipboard;
mod pkexec;
mod svg;
mod tailscale;
mod tray;

use crate::tray::menu::SysTray;
use std::thread::park;

fn main() {
    // initialize logger
    env_logger::init();

    // start the tray service
    let handle = ksni::spawn(SysTray {
        ctx: tailscale::status::get_current_status(),
    })
    .unwrap();

    // keep the main thread alive
    // NOTE: The documentation for park reads:
    // "A call to park does not guarantee that the thread will
    // remain parked forever, and callers should be prepared for this possibility."
    // hence the loop
    loop {
        park();
    }
}
