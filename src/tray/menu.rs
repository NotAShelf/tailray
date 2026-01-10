use std::{error::Error, fmt};

use log::{debug, error, info};
use muda::{Menu, MenuId, MenuItem, PredefinedMenuItem, Submenu};
use notify_rust::Notification;

use crate::{
  elevation::run_with_elevation,
  error::AppError,
  svg::renderer::Theme,
  tailscale::{
    peer::copy_peer_ip,
    status::{Status, get_current},
    utils::PeerKind,
  },
};

/// Custom error type for `SystemTray` operations
#[derive(Debug)]
pub enum TrayError {
  Command(String),
  StatusUpdate(String),
  Notification(String),
}

impl fmt::Display for TrayError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Command(msg) => write!(f, "Command execution failed: {msg}"),
      Self::StatusUpdate(msg) => write!(f, "Status update failed: {msg}"),
      Self::Notification(msg) => write!(f, "Notification failed: {msg}"),
    }
  }
}

impl Error for TrayError {}

/// Represents the context for the system tray
#[derive(Debug)]
pub struct Context {
  pub ip:     String,
  pub status: Status,
  pub theme:  Theme,
}

impl Default for Context {
  fn default() -> Self {
    Self {
      ip:     String::default(),
      status: Status::default(),
      theme:  Theme::from_env(),
    }
  }
}

/// The main `SystemTray` implementation
#[derive(Debug)]
pub struct SysTray {
  pub ctx: Context,
}

impl SysTray {
  /// Returns whether Tailscale is enabled
  pub const fn enabled(&self) -> bool {
    self.ctx.status.tailscale_up
  }

  /// Helper function to show notifications with consistent error handling
  fn show_notification(
    summary: &str,
    body: &str,
    icon: &str,
  ) -> Result<(), AppError> {
    Notification::new()
      .summary(summary)
      .body(body)
      .icon(icon)
      .show()
      .map(|_| ())
      .map_err(|e| {
        error!("Failed to show notification: {e}");
        AppError::Tray(TrayError::Notification(e.to_string()))
      })
  }

  /// Updates the Tailscale status
  pub fn update_status(&mut self) -> Result<(), AppError> {
    match get_current() {
      Ok(ctx) => {
        self.ctx = ctx;
        Ok(())
      },
      Err(e) => {
        error!("Failed to update status: {e}");
        Err(AppError::Tray(TrayError::StatusUpdate(e.to_string())))
      },
    }
  }

  /// Executes a Tailscale service command (up/down)
  pub fn do_service_link(&mut self, verb: &str) -> Result<(), AppError> {
    match run_with_elevation("tailscale", &[verb]) {
      Ok(_) => {
        info!("Link {}: success", verb);

        Self::show_notification(
          &format!("Connection {verb}"),
          &format!(
            "Tailscale service {}",
            if verb == "up" { "online" } else { "offline" }
          ),
          "info",
        )?;
        self.update_status()?;

        Ok(())
      },
      Err(e) => {
        error!("Failed to execute command: {e}");

        Self::show_notification(
          "Connection Failed",
          &format!("Failed to {verb} Tailscale: {e}"),
          "error",
        )?;

        Err(AppError::Tray(TrayError::Command(e.to_string())))
      },
    }
  }

  /// Builds the tray menu with identical structure to the ksni version
  #[allow(clippy::too_many_lines)]
  pub fn build_menu(&self) -> Result<Menu, Box<dyn Error>> {
    let menu = Menu::new();
    let device_name = &self.ctx.status.this_machine.display_name;
    let message = format!("This device: {} ({})", device_name, self.ctx.ip);
    debug!("Creating menu with device {message}");

    // Connect menu item
    menu.append(&MenuItem::with_id(
      MenuId::new("connect"),
      "Connect",
      !self.enabled(),
      None,
    ))?;

    // Disconnect menu item
    menu.append(&MenuItem::with_id(
      MenuId::new("disconnect"),
      "Disconnect",
      self.enabled(),
      None,
    ))?;

    // Separator
    menu.append(&PredefinedMenuItem::separator())?;

    // This device menu item
    menu.append(&MenuItem::with_id(
      MenuId::new("this_device"),
      &message,
      true,
      None,
    ))?;

    // Network devices submenu
    let network_devices_menu =
      Submenu::with_id("network_devices", "Network Devices", true);

    // My Devices submenu
    let my_devices_submenu = Submenu::with_id("my_devices", "My Devices", true);
    // Tailscale Services submenu
    let services_submenu =
      Submenu::with_id("tailscale_services", "Tailscale Services", true);

    // Populate device submenus
    for (_, peer) in self
      .ctx
      .status
      .peers
      .iter()
      .filter(|(_, peer)| !peer.ips.is_empty())
    {
      let ip = &peer.ips[0];
      let name = &peer.display_name;
      let peer_label = format!("{name}\t({ip})");
      let peer_id = format!("peer_{}", ip.replace(['.', ':'], "_"));

      let peer_item =
        MenuItem::with_id(MenuId::new(&peer_id), peer_label, true, None);

      match name {
        PeerKind::HostName(_) => {
          my_devices_submenu.append(&peer_item)?;
        },
        PeerKind::DNSName(_) => {
          services_submenu.append(&peer_item)?;
        },
      }
    }

    network_devices_menu.append(&my_devices_submenu)?;
    network_devices_menu.append(&services_submenu)?;
    menu.append(&network_devices_menu)?;

    // Admin Console menu item
    menu.append(&MenuItem::with_id(
      MenuId::new("admin_console"),
      "Admin Console",
      true,
      None,
    ))?;

    // Separator
    menu.append(&PredefinedMenuItem::separator())?;

    // Exit menu item
    menu.append(&MenuItem::with_id(
      MenuId::new("exit"),
      "Exit Tailray",
      true,
      None,
    ))?;

    Ok(menu)
  }

  /// Gets menu event handler for the given menu ID
  pub fn handle_menu_event(&mut self, menu_id: &str) -> Result<(), AppError> {
    match menu_id {
      "connect" => {
        if let Err(e) = self.do_service_link("up") {
          error!("Failed to connect: {e}");
          return Err(e);
        }
      },
      "disconnect" => {
        if let Err(e) = self.do_service_link("down") {
          error!("Failed to disconnect: {e}");
          return Err(e);
        }
      },
      "this_device" => {
        let message = format!(
          "This device: {} ({})",
          self.ctx.status.this_machine.display_name, self.ctx.ip
        );
        if let Err(e) = copy_peer_ip(&self.ctx.ip, &message, true) {
          error!("Failed to copy IP for this device: {e}");
        }
      },
      "admin_console" => {
        let admin_url =
          std::env::var("TAILRAY_ADMIN_URL").unwrap_or_else(|_| {
            "https://login.tailscale.com/admin/machines".to_string()
          });

        if let Err(e) = open::that(&admin_url) {
          error!("Failed to open admin console: {e}");
        }
      },
      "exit" => {
        info!("Exit menu item clicked, shutting down");
        std::process::exit(0);
      },
      id if id.starts_with("peer_") => {
        // Handle peer click - find the peer by reconstructing the IP from the
        // ID
        if let Some((_, peer)) = self.ctx.status.peers.iter().find(|(_, p)| {
          !p.ips.is_empty()
            && p.ips[0].replace(['.', ':'], "_")
              == id.strip_prefix("peer_").unwrap_or("")
        }) {
          let ip = &peer.ips[0];
          let name = &peer.display_name;
          let peer_title = format!("{name} ({ip})");
          if let Err(e) = copy_peer_ip(ip, &peer_title, false) {
            error!("Failed to copy peer IP: {e}");
          }
        }
      },
      _ => {
        debug!("Unhandled menu event: {menu_id}");
      },
    }
    Ok(())
  }
}
