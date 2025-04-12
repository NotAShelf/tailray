mod clipboard;
mod pkexec;
mod svg;
mod tailscale;
mod tray;

use crate::tray::utils::start_tray_service;
use log::{error, info};
use std::process::exit;

fn main() {
    // Initialize logger with reasonable defaults
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp(Some(env_logger::TimestampPrecision::Seconds))
        .init();

    info!("Starting Tailray application");

    // Start tray service
    if let Err(e) = start_tray_service() {
        error!("Tray service error: {}", e);
        exit(1);
    }

    // If we reach this point, the tray service has exited
    info!("Tailray shutting down");
}
