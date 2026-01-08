use std::fmt;

/// Errors that can occur during command elevation operations.
///
/// This enum represents various failure modes when attempting to run
/// commands with elevated privileges on different platforms.
#[derive(Debug, Clone, PartialEq)]
pub enum ElevationError {
  /// Failed to spawn the elevation helper or the target command.
  ///
  /// This typically indicates an issue with the elevation tool itself
  /// (e.g., pkexec, sudo, or osascript not found or not working).
  ExecutionFailed {
    /// The program that failed to execute (either elevation helper or target)
    program: String,

    /// Human-readable reason for the failure
    reason: String,
  },

  /// The elevation tool executed but the command failed with a non-zero exit
  /// code.
  CommandFailed {
    /// The target program that was executed
    program: String,

    /// The exit code returned by the command
    code: i32,

    /// Error message from stderr or other output
    message: String,
  },

  /// No suitable elevation tool was found on the platform.
  ElevationToolNotFound {
    /// The platform where no tool was found (e.g., "Linux")
    platform: String,

    /// List of tools that were searched for but not available
    tools: Vec<String>,
  },

  /// The user denied the elevation request (e.g., clicked "No" on UAC prompt).
  ElevationDenied,

  /// The current environment does not support elevation.
  ///
  /// This can occur on headless Linux systems without DISPLAY/WAYLAND_DISPLAY
  /// and no TTY, or in systemd user services without a controlling terminal.
  UnsupportedEnvironment {
    /// The platform where elevation is unsupported
    platform: String,

    /// Description of the constraint preventing elevation
    constraint: String,
  },
}

impl fmt::Display for ElevationError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::ExecutionFailed { program, reason } => {
        write!(f, "Failed to execute '{}': {}", program, reason)
      },

      Self::CommandFailed {
        program,
        code,
        message,
      } => {
        write!(
          f,
          "Command '{}' failed with code {}: {}",
          program, code, message
        )
      },

      Self::ElevationToolNotFound { platform, tools } => {
        write!(
          f,
          "No elevation tool found on {}: tried {}",
          platform,
          tools.join(", ")
        )
      },

      Self::ElevationDenied => {
        write!(f, "Elevation was denied by the user")
      },

      Self::UnsupportedEnvironment {
        platform,
        constraint,
      } => {
        write!(f, "Cannot elevate on {}: {}", platform, constraint)
      },
    }
  }
}

impl std::error::Error for ElevationError {}
