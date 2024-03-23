mod clipboard;
mod pkexec;
mod svg;
mod tailscale;
mod tray;

use crate::tray::utils::start_tray_service;
use std::thread::park;

fn main() {
    // initialize logger
    env_logger::init();

    // start tray service
    start_tray_service();

    // keep the main thread alive
    loop {
        park();
    }
}
