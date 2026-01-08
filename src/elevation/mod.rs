pub mod detection;
pub mod error;
pub mod linux;
pub mod macos;
pub mod windows;

use std::process::{Command, Stdio};

pub use detection::{Platform, check_elevation_required};
pub use error::ElevationError;
pub use linux::elevate_command_linux;
pub use macos::elevate_command_macos;
pub use windows::elevate_command_windows;

/// Attempts to run a command, elevating if necessary
pub fn run_with_elevation(
  program: &str,
  args: &[&str],
) -> Result<String, ElevationError> {
  if check_elevation_required(program, args)? {
    run_elevated(program, args)
  } else {
    run_direct(program, args)
  }
}

/// Runs command without elevation
fn run_direct(program: &str, args: &[&str]) -> Result<String, ElevationError> {
  let output = Command::new(program)
    .args(args)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .and_then(|child| child.wait_with_output())
    .map_err(|e| {
      ElevationError::ExecutionFailed {
        program: program.to_string(),
        reason:  e.to_string(),
      }
    })?;

  if output.status.success() {
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
  } else {
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(ElevationError::CommandFailed {
      program: program.to_string(),
      code:    output.status.code().unwrap_or(-1),
      message: stderr.into_owned(),
    })
  }
}

/// Runs command with elevation using platform-appropriate method
fn run_elevated(
  program: &str,
  args: &[&str],
) -> Result<String, ElevationError> {
  match Platform::current() {
    Platform::Linux => elevate_command_linux(program, args),
    Platform::MacOS => elevate_command_macos(program, args),
    Platform::Windows => elevate_command_windows(program, args),
    Platform::BSD => {
      // BSD systems use sudo like Linux
      elevate_command_linux(program, args)
    },
  }
}
