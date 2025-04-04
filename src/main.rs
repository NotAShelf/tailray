mod clipboard;
mod pkexec;
mod svg;
mod tailscale;
mod tray;

use crate::tray::utils::start_tray_service;
use log::{error, info};
use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() {
    // Initialize logger with reasonable defaults
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp(Some(env_logger::TimestampPrecision::Seconds))
        .init();

    info!("Starting Tailray application");

    // Set up graceful shutdown handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    if let Err(e) = ctrlc::set_handler(move || {
        info!("Received shutdown signal, exiting...");
        r.store(false, Ordering::SeqCst);
    }) {
        error!("Failed to set Ctrl+C handler: {}", e);
        exit(1);
    }

    // Start tray service
    match start_tray_service() {
        Ok(()) => info!("Tray service started successfully"),
        Err(e) => {
            error!("Failed to start the tray service: {}", e);
            exit(1);
        }
    }

    // Keep the main thread alive while handling signals
    info!("Application running, press Ctrl+C to exit");
    while running.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    info!("Tailray shutting down");
}
