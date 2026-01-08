use std::{env, io::IsTerminal};

/// Supported platforms for elevation operations.
///
/// This enum identifies the target platform to select the appropriate
/// elevation mechanism (pkexec/sudo for Linux, osascript for macOS,
/// UAC for Windows).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
  /// Linux operating systems
  Linux,

  /// Apple macOS
  MacOS,

  /// Microsoft Windows
  Windows,

  /// BSD family (FreeBSD, OpenBSD, NetBSD, DragonFly)
  #[allow(clippy::upper_case_acronyms)]
  BSD,
}

impl Platform {
  /// Detects the current platform at runtime based on the target OS.
  ///
  /// # Returns
  ///
  /// Returns the appropriate [`Platform`] variant matching the compiled
  /// target operating system.
  ///
  /// # Examples
  ///
  /// ```
  /// use crate::elevation::detection::Platform;
  ///
  /// let current = Platform::current();
  /// ```
  pub fn current() -> Self {
    if cfg!(target_os = "linux") {
      Self::Linux
    } else if cfg!(target_os = "macos") {
      Self::MacOS
    } else if cfg!(target_os = "windows") {
      Self::Windows
    } else if cfg!(any(
      target_os = "freebsd",
      target_os = "openbsd",
      target_os = "netbsd",
      target_os = "dragonfly"
    )) {
      Self::BSD
    } else {
      // Default to Linux for other Unix-like systems (muslin, etc.)
      Self::Linux
    }
  }
}

/// Determines whether a command requires elevated privileges to execute.
///
/// This function performs platform-specific checks to determine if elevation
/// is needed. For tailscale commands, it checks socket accessibility. On Linux,
/// it also verifies the environment supports elevation (not a headless system
/// or systemd user service without TTY).
///
/// # Arguments
///
/// * `program` - The name or path of the program to execute
/// * `args` - Command-line arguments to pass to the program
///
/// # Returns
///
/// * `Ok(true)` - Elevation is required to run the command
/// * `Ok(false)` - Elevation is not required
/// * `Err(ElevationError::UnsupportedEnvironment)` - Cannot elevate in current
///   environment (e.g., headless Linux without DISPLAY/WAYLAND_DISPLAY and no
///   TTY)
/// * `Err(ElevationError::ElevationToolNotFound)` - No elevation tool available
///   on the platform
pub fn check_elevation_required(
  program: &str,
  args: &[&str],
) -> Result<bool, crate::elevation::error::ElevationError> {
  // Special handling for tailscale commands
  if program == "tailscale" {
    return check_tailscale_elevation(args);
  }

  // On Linux, check if we're running in a terminal session
  // pkexec/sudo need a TTY or graphical session for password prompts
  if Platform::current() == Platform::Linux {
    // Check for a controlling terminal (TTY or graphical session)
    let has_display =
      env::var("DISPLAY").is_ok() || env::var("WAYLAND_DISPLAY").is_ok();
    let has_tty = env::var("XDG_SESSION_TYPE")
      .map(|t| t == "tty")
      .unwrap_or(false)
      || std::io::stdin().is_terminal();

    // In systemd user services without a terminal, elevation is not available
    if env::var("INVOCATION_ID").is_ok() && !has_display && !has_tty {
      return Err(
        crate::elevation::error::ElevationError::UnsupportedEnvironment {
          platform:   String::from("Linux"),
          constraint: String::from(
            "systemd user service without controlling terminal",
          ),
        },
      );
    }

    // Check if we have a graphical session (X11 or Wayland) or a TTY for
    // pkexec/sudo
    if !has_display && !has_tty {
      return Err(
        crate::elevation::error::ElevationError::UnsupportedEnvironment {
          platform:   String::from("Linux"),
          constraint: String::from(
            "headless environment without display or TTY",
          ),
        },
      );
    }
  }

  // For other commands, assume elevation is not needed
  Ok(false)
}

/// Checks if a tailscale command requires elevated privileges.
///
/// Tailscale's `up` command may require root access depending on socket
/// permissions. Other tailscale commands typically don't require elevation.
///
/// # Arguments
///
/// * `args` - Command-line arguments passed to tailscale
///
/// # Returns
///
/// * `Ok(true)` - Elevation is required
/// * `Ok(false)` - Elevation is not required
fn check_tailscale_elevation(
  args: &[&str],
) -> Result<bool, crate::elevation::error::ElevationError> {
  // `tailscale up` typically requires root on some systems
  if args.contains(&"up") {
    match Platform::current() {
      Platform::Linux => {
        // Check if we can access the socket on any Linux distro
        match can_access_tailscale_socket() {
          Ok(can_access) => Ok(!can_access),
          Err(_) => Ok(true), // If check fails, assume elevation needed
        }
      },

      Platform::MacOS | Platform::Windows | Platform::BSD => {
        // macOS, Windows, and BSD typically don't need sudo for tailscale up
        Ok(false)
      },
    }
  } else if args.contains(&"down") {
    // tailscale down requires root if socket is not accessible
    match Platform::current() {
      Platform::Linux => {
        match can_access_tailscale_socket() {
          Ok(can_access) => Ok(!can_access),
          Err(_) => Ok(true), // if check fails, assume elevation needed
        }
      },
      Platform::MacOS | Platform::Windows | Platform::BSD => {
        // macOS, Windows, and BSD typically don't need sudo for tailscale down
        Ok(false)
      },
    }
  } else {
    // Other tailscale commands
    Ok(false)
  }
}

/// Attempts to connect to the Tailscale control socket to verify accessibility.
///
/// Checks if the current user can access the tailscaled socket without root
/// privileges. The socket path varies by platform:
/// - Linux: `/var/run/tailscale/tailscaled.sock`
/// - macOS: `/var/run/tailscale/tailscaled.sock` (or via launchd)
/// - BSD: Platform-specific path
///
/// # Returns
///
/// * `Ok(true)` - Socket is accessible, no elevation needed
/// * `Ok(false)` - Socket is not accessible (not found or permission denied)
/// * `Err(())` - Unable to perform the check (unexpected error)
fn can_access_tailscale_socket() -> Result<bool, ()> {
  #[cfg(unix)]
  {
    use std::os::unix::net::UnixStream;

    let socket_path = tailscale_socket_path();

    match UnixStream::connect(&socket_path) {
      Ok(_stream) => Ok(true),
      Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
      Err(_) => {
        // Permission denied or other error - can't access socket
        Ok(false)
      },
    }
  }

  #[cfg(not(unix))]
  {
    Ok(false)
  }
}

/// Returns the platform-specific Tailscale socket path.
///
/// # Returns
///
/// The filesystem path to the tailscaled control socket.
#[cfg(unix)]
fn tailscale_socket_path() -> std::path::PathBuf {
  #[cfg(target_os = "linux")]
  {
    std::path::PathBuf::from("/var/run/tailscale/tailscaled.sock")
  }
  #[cfg(target_os = "macos")]
  {
    std::path::PathBuf::from("/var/run/tailscale/tailscaled.sock")
  }
  #[cfg(any(
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
  ))]
  {
    std::path::PathBuf::from("/var/run/tailscaled.sock")
  }
  #[cfg(not(any(target_os = "linux", target_os = "macos")))]
  {
    std::path::PathBuf::from("/var/run/tailscale/tailscaled.sock")
  }
}
