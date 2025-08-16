use crate::tailscale;
use crate::tray::menu::SysTray;

use ksni::blocking::TrayMethods;
use log::{error, info};
use std::{
    error::Error,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

type TrayServiceError = Box<dyn Error>;

// Global flag for tracking disconnection
static NEEDS_RESPAWN: AtomicBool = AtomicBool::new(false);

pub fn start_tray_service() -> Result<(), TrayServiceError> {
    // Get initial status for the tray
    let status = tailscale::status::get_current()
        .map_err(|e| format!("Failed to update Tailscale status: {e}"))?;

    // Initial tray spawning
    let handle = SysTray { ctx: status }
        .spawn()
        .map_err(|e| format!("Failed to spawn Tray implementation: {e}"))?;

    // Flag to control application lifecycle
    let running = Arc::new(AtomicBool::new(true));

    // Set up signal handler for graceful shutdown
    {
        let running = running.clone();
        ctrlc::set_handler(move || {
            info!("Received shutdown signal, exiting...");
            running.store(false, Ordering::SeqCst);
        })
        .map_err(|e| format!("Failed to set Ctrl+C handler: {e}"))?;
    }

    // Start watchdog thread for spawning new trays if status bar restarts
    {
        let running = running.clone();
        thread::spawn(move || {
            info!("Started watchdog thread for status bar monitoring");
            let mut consecutive_failures = 0;
            let mut handle = Some(handle);

            while running.load(Ordering::SeqCst) {
                if NEEDS_RESPAWN.load(Ordering::SeqCst) || handle.is_none() {
                    info!("Tray disconnected or missing, attempting to respawn...");
                    NEEDS_RESPAWN.store(false, Ordering::SeqCst);

                    match tailscale::status::get_current()
                        .and_then(|ctx| SysTray { ctx }.spawn().map_err(std::convert::Into::into))
                    {
                        Ok(new_handle) => {
                            info!("Successfully respawned tray icon");
                            handle = Some(new_handle);
                            consecutive_failures = 0;
                        }
                        Err(e) => {
                            consecutive_failures += 1;
                            error!("Failed to respawn tray (attempt {consecutive_failures}): {e}");
                        }
                    }

                    if consecutive_failures > 0 {
                        let delay = (2_u64.pow(consecutive_failures.min(4))).min(30);
                        thread::sleep(Duration::from_secs(delay));
                    }
                }
                thread::sleep(Duration::from_secs(2));
            }
            info!("Watchdog thread exiting");
        });
    }

    // Keep the main thread alive until shutdown is requested
    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(100));
    }

    info!("Tray service shutting down");
    Ok(())
}

// Function to signal that the tray needs to be respawned
pub fn signal_respawn_needed() {
    NEEDS_RESPAWN.store(true, Ordering::SeqCst);
}
