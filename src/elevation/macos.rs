use std::process::{Command, Stdio};

use super::error::ElevationError;

/// Escapes a shell command string for embedding in an AppleScript string
/// literal.
///
/// The AppleScript `do shell script` command takes a shell command as a string.
/// This function escapes special characters (backslashes, double quotes,
/// newlines) so the shell command is correctly passed through AppleScript's
/// string parsing.
///
/// # Arguments
///
/// * `s` - The shell command string to escape
///
/// # Returns
///
/// The escaped string wrapped in AppleScript double quotes
fn escape_for_applescript(s: &str) -> String {
  let escaped = s
    .replace("\\", "\\\\")
    .replace("\"", "\\\"")
    .replace("\n", "\\n")
    .replace("\r", "\\r")
    .replace("\t", "\\t");
  format!("\"{}\"", escaped)
}

/// Elevates a command on macOS using `osascript` to invoke AppleScript.
///
/// This function uses AppleScript's `do shell script with administrator
/// privileges` to prompt the user for credentials via the system's
/// authentication dialog. Arguments are shell-escaped using `shlex` before
/// being embedded in the command.
///
/// # Arguments
///
/// * `program` - Path or name of the program to execute
/// * `args` - Command-line arguments to pass to the program
///
/// # Returns
///
/// * `Ok(String)` - Command stdout on success
/// * `Err(ElevationError::ExecutionFailed)` - Failed to spawn osascript or
///   shell error
/// * `Err(ElevationError::CommandFailed)` - Elevated command returned non-zero
///   exit code
pub fn elevate_command_macos(
  program: &str,
  args: &[&str],
) -> Result<String, ElevationError> {
  // On macOS, tailscale typically doesn't require sudo for up/down
  // But if elevation is needed, we use osascript with admin privileges

  // Shell-escape each argument using shlex, then join into a command string
  let program_quoted = shlex::try_quote(program)
    .map_err(|e| {
      ElevationError::ExecutionFailed {
        program: program.to_string(),
        reason:  format!("Shell quoting failed: {}", e),
      }
    })?
    .to_string();
  let args_quoted: Result<Vec<String>, _> = args
    .iter()
    .map(|a| shlex::try_quote(a).map(String::from))
    .collect();
  let args_quoted = args_quoted.map_err(|e| {
    ElevationError::ExecutionFailed {
      program: program.to_string(),
      reason:  format!("Shell quoting failed: {}", e),
    }
  })?;

  let mut parts: Vec<&str> = vec![];
  parts.push(&program_quoted);
  for arg in &args_quoted {
    parts.push(arg);
  }

  let cmd_str = shlex::try_join(parts).map_err(|e| {
    ElevationError::ExecutionFailed {
      program: program.to_string(),
      reason:  format!("Shell join failed: {}", e),
    }
  })?;

  // Escape the entire command string for AppleScript and pass to do shell
  // script
  let escaped_cmd = escape_for_applescript(&cmd_str);
  let script = format!(
    "do shell script {} with administrator privileges",
    escaped_cmd
  );

  let output = Command::new("osascript")
    .arg("-e")
    .arg(&script)
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
