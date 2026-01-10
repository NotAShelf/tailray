use std::{
  cell::RefCell,
  error::Error,
  rc::Rc,
  sync::{Arc, Mutex},
  time::Duration,
};

use log::{error, info};
use muda::MenuEvent;
use tray_icon::{TrayIcon, TrayIconBuilder};

use crate::{svg::renderer::Resvg, tailscale, tray::menu::SysTray};

type TrayServiceError = Box<dyn Error>;

/// Builds and returns a configured tray icon with menu and icon
fn build_tray_icon(
  tray_context: &Arc<Mutex<SysTray>>,
) -> Result<TrayIcon, TrayServiceError> {
  let ctx = tray_context
    .lock()
    .map_err(|e| format!("Failed to lock tray context: {e}"))?;

  let menu = ctx.build_menu()?;
  let icon = Resvg::load_icon(ctx.ctx.theme, ctx.enabled())
    .ok_or("Failed to load tray icon")?;

  TrayIconBuilder::new()
    .with_menu(Box::new(menu))
    .with_tooltip("Tailray - Tailscale System Tray")
    .with_icon(icon)
    .build()
    .map_err(|e| format!("Failed to build tray icon: {e}").into())
}

/// Updates the tray icon and menu based on current context
fn update_tray(
  tray: &mut TrayIcon,
  ctx: &mut SysTray,
) -> Result<(), Box<dyn Error>> {
  let new_menu = ctx.build_menu()?;
  tray.set_menu(Some(Box::new(new_menu)));

  if let Some(new_icon) = Resvg::load_icon(ctx.ctx.theme, ctx.enabled()) {
    tray.set_icon(Some(new_icon))?;
  }

  Ok(())
}

/// Handles menu events and updates the tray accordingly
fn handle_menu_event(
  menu_id: &str,
  tray_context: &Arc<Mutex<SysTray>>,
  tray_icon: &Rc<RefCell<TrayIcon>>,
) {
  let mut ctx = match tray_context.lock() {
    Ok(ctx) => ctx,
    Err(e) => {
      error!("Failed to lock tray context: {e}");
      return;
    },
  };

  if let Err(e) = ctx.handle_menu_event(menu_id) {
    error!("Failed to handle menu event '{menu_id}': {e}");
    return;
  }

  // Update tray after handling event
  if let Ok(mut tray) = tray_icon.try_borrow_mut()
    && let Err(e) = update_tray(&mut tray, &mut ctx)
  {
    error!("Failed to update tray: {e}");
  }
}

/// Performs periodic status updates and refreshes the tray icon
fn update_status(
  tray_context: &Arc<Mutex<SysTray>>,
  tray_icon: &Rc<RefCell<TrayIcon>>,
) {
  let mut ctx = match tray_context.lock() {
    Ok(ctx) => ctx,
    Err(e) => {
      error!("Failed to lock tray context: {e}");
      return;
    },
  };

  if let Err(e) = ctx.update_status() {
    error!("Failed to update Tailscale status: {e}");
    return;
  }

  // Update icon after status update
  if let Ok(mut tray) = tray_icon.try_borrow_mut()
    && let Err(e) = update_tray(&mut tray, &mut ctx)
  {
    error!("Failed to update tray icon: {e}");
  }
}

#[cfg(target_os = "linux")]
fn run_linux_event_loop(
  tray_icon: TrayIcon,
  tray_context: Arc<Mutex<SysTray>>,
) {
  use gtk::glib;

  let tray_icon = Rc::new(RefCell::new(tray_icon));
  let menu_channel = MenuEvent::receiver();

  // Process menu events every 100ms
  {
    let tray_context = tray_context.clone();
    let tray_icon = tray_icon.clone();

    glib::timeout_add_local(Duration::from_millis(100), move || {
      if let Ok(event) = menu_channel.try_recv() {
        let menu_id = event.id().0.clone();
        handle_menu_event(&menu_id, &tray_context, &tray_icon);
      }
      glib::ControlFlow::Continue
    });
  }

  // Check for status updates every second (update every 10s)
  {
    let tray_context = tray_context.clone();
    let tray_icon = tray_icon.clone();
    let last_update = Rc::new(RefCell::new(std::time::Instant::now()));

    glib::timeout_add_local(Duration::from_secs(1), move || {
      let elapsed = last_update.borrow().elapsed();
      if elapsed >= Duration::from_secs(10) {
        update_status(&tray_context, &tray_icon);
        *last_update.borrow_mut() = std::time::Instant::now();
      }
      glib::ControlFlow::Continue
    });
  }

  info!("Starting GTK main loop");
  gtk::main();
}

#[cfg(not(target_os = "linux"))]
fn run_event_loop(mut tray_icon: TrayIcon, tray_context: Arc<Mutex<SysTray>>) {
  use std::sync::atomic::{AtomicBool, Ordering};

  let running = Arc::new(AtomicBool::new(true));
  let menu_channel = MenuEvent::receiver();
  let mut last_update = std::time::Instant::now();

  info!("Starting event loop");

  while running.load(Ordering::SeqCst) {
    // Process menu events
    if let Ok(event) = menu_channel.try_recv() {
      let menu_id = event.id().0.clone();

      let mut ctx = match tray_context.lock() {
        Ok(ctx) => ctx,
        Err(e) => {
          error!("Failed to lock tray context: {e}");
          continue;
        },
      };

      if let Err(e) = ctx.handle_menu_event(&menu_id) {
        error!("Failed to handle menu event '{menu_id}': {e}");
      }

      if let Err(e) = update_tray(&mut tray_icon, &mut ctx) {
        error!("Failed to update tray: {e}");
      }
    }

    // Status updates every 10 seconds
    if last_update.elapsed() >= Duration::from_secs(10) {
      let mut ctx = match tray_context.lock() {
        Ok(ctx) => ctx,
        Err(e) => {
          error!("Failed to lock tray context: {e}");
          continue;
        },
      };

      if let Err(e) = ctx.update_status() {
        error!("Failed to update status: {e}");
      } else if let Err(e) = update_tray(&mut tray_icon, &mut ctx) {
        error!("Failed to update tray icon: {e}");
      }

      last_update = std::time::Instant::now();
    }

    std::thread::sleep(Duration::from_millis(100));
  }
}

pub fn start_tray_service() -> Result<(), TrayServiceError> {
  info!("Initializing Tailray tray service");

  // Initialize GTK on Linux
  #[cfg(target_os = "linux")]
  {
    gtk::init().map_err(|e| format!("Failed to initialize GTK: {e}"))?;
    info!("GTK initialized successfully");
  }

  // Get initial Tailscale status
  let status = tailscale::status::get_current()
    .map_err(|e| format!("Failed to get Tailscale status: {e}"))?;

  info!(
    "Tailscale status: {} ({})",
    if status.status.tailscale_up {
      "connected"
    } else {
      "disconnected"
    },
    status.status.this_machine.display_name
  );

  // Create tray context
  let tray_context = Arc::new(Mutex::new(SysTray { ctx: status }));

  // Build tray icon
  let tray_icon = build_tray_icon(&tray_context)?;
  info!("Tray icon created successfully");

  // Set up graceful shutdown handler
  #[cfg(target_os = "linux")]
  {
    // On Linux, use glib::idle_add to quit from main thread
    ctrlc::set_handler(move || {
      info!("Received shutdown signal, exiting gracefully");
      gtk::glib::idle_add_once(|| {
        gtk::main_quit();
      });
    })
    .map_err(|e| format!("Failed to set Ctrl+C handler: {e}"))?;
  }

  #[cfg(not(target_os = "linux"))]
  {
    ctrlc::set_handler(move || {
      info!("Received shutdown signal, exiting gracefully");
      std::process::exit(0);
    })
    .map_err(|e| format!("Failed to set Ctrl+C handler: {e}"))?;
  }

  // Run platform-specific event loop
  #[cfg(target_os = "linux")]
  run_linux_event_loop(tray_icon, tray_context);

  #[cfg(not(target_os = "linux"))]
  run_event_loop(tray_icon, tray_context);

  info!("Tray service shutting down");
  Ok(())
}
