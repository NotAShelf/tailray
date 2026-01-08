use std::process::{Command, Stdio};

use super::error::ElevationError;

/// Elevates a command on Linux using the appropriate privilege escalation tool.
///
/// First checks for `pkexec` (Polkit) availability, falling back
/// to `sudo` if pkexec is not available. Both tools require a graphical session
/// or TTY for password prompts.
///
/// # Arguments
///
/// * `program` - Path or name of the program to execute with elevation
/// * `args` - Command-line arguments to pass to the program
///
/// # Returns
///
/// * `Ok(String)` - Command output on success
/// * `Err(ElevationError::ElevationToolNotFound)` - Neither pkexec nor sudo
///   available
/// * `Err(ElevationError::ElevationDenied)` - User denied the elevation request
/// * `Err(ElevationError::ExecutionFailed)` - Failed to spawn the elevation
///   helper
/// * `Err(ElevationError::CommandFailed)` - Elevated command returned non-zero
///   exit code
pub fn elevate_command_linux(
  program: &str,
  args: &[&str],
) -> Result<String, ElevationError> {
  // Check for pkexec (Polkit)
  if which::which("pkexec").is_ok() {
    return elevate_with_pkexec(program, args);
  }

  // Check for sudo
  if which::which("sudo").is_ok() {
    return elevate_with_sudo(program, args);
  }

  Err(ElevationError::ElevationToolNotFound {
    platform: String::from("Linux"),
    tools:    vec![String::from("pkexec"), String::from("sudo")],
  })
}

/// Executes a command using `pkexec` for privilege elevation.
///
/// pkexec is the Polkit authentication agent that prompts the user with a
/// graphical dialog for credentials. Exit code 126 indicates the user denied
/// the request.
///
/// # Arguments
///
/// * `program` - Path or name of the program to execute
/// * `args` - Command-line arguments to pass to the program
///
/// # Returns
///
/// * `Ok(String)` - Command stdout on success
/// * `Err(ElevationError::ExecutionFailed)` - Failed to spawn pkexec
/// * `Err(ElevationError::ElevationDenied)` - User denied (exit code 126)
/// * `Err(ElevationError::CommandFailed)` - Command failed with other exit code
fn elevate_with_pkexec(
  program: &str,
  args: &[&str],
) -> Result<String, ElevationError> {
  let mut cmd = Command::new("pkexec");
  cmd.arg(program).args(args);

  // pkexec needs a terminal for password prompt
  // In GUI apps, we need to use a graphical polkit agent
  // We'll let polkit handle the prompt
  cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

  let output = cmd
    .spawn()
    .map_err(|e| {
      ElevationError::ExecutionFailed {
        program: "pkexec".to_string(),
        reason:  e.to_string(),
      }
    })?
    .wait_with_output()
    .map_err(|e| {
      ElevationError::ExecutionFailed {
        program: program.to_string(),
        reason:  e.to_string(),
      }
    })?;

  if output.status.success() {
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
  } else if output.status.code() == Some(126) {
    Err(ElevationError::ElevationDenied)
  } else {
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(ElevationError::CommandFailed {
      program: program.to_string(),
      code:    output.status.code().unwrap_or(-1),
      message: stderr.into_owned(),
    })
  }
}

/// Executes a command using `sudo` for privilege elevation.
///
/// This function first attempts passwordless sudo with the `-n` flag. If that
/// fails due to authentication requirements, it falls back to interactive sudo.
/// Setting `LC_ALL=C` and `LANG=C` ensures consistent, English error messages
/// for reliable parsing of authentication failures.
///
/// # Arguments
///
/// * `program` - Path or name of the program to execute
/// * `args` - Command-line arguments to pass to the program
///
/// # Returns
///
/// * `Ok(String)` - Command stdout on success
/// * `Err(ElevationError::ExecutionFailed)` - Failed to spawn sudo
/// * `Err(ElevationError::CommandFailed)` - Elevated command failed
///
/// # Note
///
/// This may execute the command twice if passwordless sudo is not available.
/// For idempotent commands (like `tailscale up/down`), this is safe.
fn elevate_with_sudo(
  program: &str,
  args: &[&str],
) -> Result<String, ElevationError> {
  // Try passwordless sudo first (non-interactive)
  let mut cmd = Command::new("sudo");
  cmd.arg("-n").arg(program).args(args);
  cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
  cmd.env("LC_ALL", "C");
  cmd.env("LANG", "C");

  if let Ok(output) = cmd
    .spawn()
    .map_err(|e| {
      ElevationError::ExecutionFailed {
        program: "sudo".to_string(),
        reason:  e.to_string(),
      }
    })
    .and_then(|child| {
      child.wait_with_output().map_err(|e| {
        ElevationError::ExecutionFailed {
          program: program.to_string(),
          reason:  e.to_string(),
        }
      })
    })
  {
    if output.status.success() {
      return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    } else {
      let stderr = String::from_utf8_lossy(&output.stderr);
      let exit_code = output.status.code().unwrap_or(-1);
      // Check for authentication-related failures using exit code and stderr
      // sudo exits 1 on auth failures
      let is_auth_failure = exit_code == 1
        || stderr.contains("a password is required")
        || stderr.contains("no tty present");
      if is_auth_failure {
        // Fall through to interactive sudo
      } else {
        // Other errors (command not found, permission denied, etc.)
        // Don't retry, return the error
        return Err(ElevationError::CommandFailed {
          program: program.to_string(),
          code:    exit_code,
          message: stderr.into_owned(),
        });
      }
    }
  }

  // Passwordless sudo not available or auth required, try interactive sudo
  let mut cmd = Command::new("sudo");
  cmd.arg(program).args(args);
  cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
  cmd.env("LC_ALL", "C");
  cmd.env("LANG", "C");

  let output = cmd
    .spawn()
    .map_err(|e| {
      ElevationError::ExecutionFailed {
        program: "sudo".to_string(),
        reason:  e.to_string(),
      }
    })?
    .wait_with_output()
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
    if stderr.contains("no tty present") {
      return Err(ElevationError::ElevationDenied);
    }
    Err(ElevationError::CommandFailed {
      program: program.to_string(),
      code:    output.status.code().unwrap_or(-1),
      message: stderr.into_owned(),
    })
  }
}
