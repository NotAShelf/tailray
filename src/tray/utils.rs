use crate::tailscale;
use crate::tray::menu::SysTray;

use ksni::blocking::TrayMethods;
use log::{error, info};
use std::{
    error::Error,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
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
    let watchdog_running = running.clone();

    // Set up signal handler for graceful shutdown
    ctrlc::set_handler({
        let handler_running = running.clone();
        move || {
            info!("Received shutdown signal, exiting...");
            handler_running.store(false, Ordering::SeqCst);
        }
    })
    .map_err(|e| format!("Failed to set Ctrl+C handler: {}", e))?;

    // Start watchdog thread for spawning new trays if status bar restarts
    thread::spawn(move || {
        info!("Started watchdog thread for status bar monitoring");

        // Track consecutive failures for backoff calculation
        let mut consecutive_failures = 0;
        let mut current_handle = Some(handle);

        // Loop until shutdown signal
        while watchdog_running.load(Ordering::SeqCst) {
            // Check if we need to respawn the tray
            let should_respawn = NEEDS_RESPAWN.load(Ordering::SeqCst);

            if should_respawn || current_handle.is_none() {
                info!("Tray disconnected or missing, attempting to respawn...");

                // Reset the flag
                NEEDS_RESPAWN.store(false, Ordering::SeqCst);

                // Try to get current status and spawn a new tray
                match tailscale::status::get_current() {
                    Ok(new_status) => {
                        // Create a new SysTray instance
                        let systray = SysTray { ctx: new_status };

                        // Try to spawn it
                        match systray.spawn() {
                            Ok(new_handle) => {
                                info!("Successfully respawned tray icon");
                                current_handle = Some(new_handle);
                                consecutive_failures = 0;
                            }
                            Err(e) => {
                                consecutive_failures += 1;
                                error!(
                                    "Failed to spawn tray (attempt {}): {}",
                                    consecutive_failures, e
                                );
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to get Tailscale status: {}", e);
                        consecutive_failures += 1;
                    }
                }

                // Exponential backoff for failures (capped at 30 seconds)
                if consecutive_failures > 0 {
                    let delay = (2_u64.pow(consecutive_failures.min(4))).min(30);
                    thread::sleep(Duration::from_secs(delay));
                }
            }

            // Check status every 2 seconds
            thread::sleep(Duration::from_secs(2));
        }

        info!("Watchdog thread exiting");
    });

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
