use std::{
  io::Write,
  process::{Command, Stdio},
};

use tempfile::NamedTempFile;

use super::error::ElevationError;

/// Escapes a string for use as a PowerShell argument using single quotes.
///
/// PowerShell uses single quotes for literal strings that don't interpret
/// escape sequences. This function wraps the argument in single quotes and
/// escapes any single quotes within the argument by doubling them.
///
/// # Arguments
///
/// * `arg` - The argument to escape
///
/// # Returns
///
/// The argument wrapped in single quotes with internal single quotes escaped
fn escape_ps_arg(arg: &str) -> String {
  format!("'{}'", arg.replace("'", "''"))
}

/// Elevates a command on Windows using UAC via PowerShell.
///
/// This function creates a temporary file for output capture, then spawns
/// an elevated PowerShell process using `Start-Process -Verb RunAs` to trigger
/// the UAC dialog. The elevated command writes its output to the temp file,
/// which is then read back.
///
/// # Arguments
///
/// * `program` - Path or name of the program to execute
/// * `args` - Command-line arguments to pass to the program
///
/// # Returns
///
/// * `Ok(String)` - Command stdout on success
/// * `Err(ElevationError::ExecutionFailed)` - Failed to create temp file or
///   spawn PowerShell
/// * `Err(ElevationError::CommandFailed)` - Elevated command returned non-zero
///   exit code
///
/// # Note
///
/// The temporary file is automatically cleaned up when the function returns
/// (via `NamedTempFile`'s Drop implementation).
pub fn elevate_command_windows(
  program: &str,
  args: &[&str],
) -> Result<String, ElevationError> {
  // Escape program and arguments for PowerShell
  let escaped_program = escape_ps_arg(program);
  let escaped_args: Vec<String> =
    args.iter().map(|a| escape_ps_arg(a)).collect();

  // Create a secure temp file for output capture
  let mut temp_file = NamedTempFile::new().map_err(|e| {
    ElevationError::ExecutionFailed {
      program: program.to_string(),
      reason:  format!("Failed to create temp file: {}", e),
    }
  })?;

  // Clone path early to avoid borrow conflicts
  let temp_file_path = temp_file.path().to_path_buf();

  // Build PowerShell script that writes output to temp file
  let temp_file_escaped =
    escape_ps_arg(temp_file_path.to_string_lossy().as_ref());
  let redirect = format!(
    "*>&1 | Out-File -FilePath {} -Encoding UTF8",
    temp_file_escaped
  );

  // Build argument string for the elevated command
  let elevated_args = if args.is_empty() {
    redirect
  } else {
    format!("{} {}", redirect, escaped_args.join(" "))
  };

  // Build the elevated PowerShell command
  let elevated_ps = format!(
    "& {} {}; exit $LASTEXITCODE",
    escaped_program, elevated_args
  );

  // Wrapper script to run elevated PowerShell with UAC prompt
  let ps_script = format!(
    "Start-Process powershell -ArgumentList '-NoProfile', '-Command', {} \
     -Verb RunAs -Wait -WindowStyle Hidden",
    escape_ps_arg(&elevated_ps)
  );

  // Initialize temp file (we write a newline to ensure file is non-empty)
  {
    writeln!(temp_file).map_err(|e| {
      ElevationError::ExecutionFailed {
        program: program.to_string(),
        reason:  format!("Failed to write to temp file: {}", e),
      }
    })?;
  }

  let output = Command::new("powershell")
    .args(["-NoProfile", "-Command", &ps_script])
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

  // Read output from temp file with retry to handle filesystem buffering
  if output.status.success() {
    let mut output_content = String::new();
    for _ in 0..10 {
      output_content =
        std::fs::read_to_string(&temp_file_path).unwrap_or_default();
      if !output_content.is_empty() {
        break;
      }
      std::thread::sleep(std::time::Duration::from_millis(50));
    }
    Ok(output_content)
  } else {
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Try to read the temp file to get the elevated command's actual output
    let elevated_output =
      std::fs::read_to_string(&temp_file_path).unwrap_or_default();

    // Use temp file content if available, otherwise stderr, otherwise a generic
    // message
    let error_message = if !elevated_output.is_empty() {
      elevated_output
    } else if !stderr.is_empty() {
      stderr.into_owned()
    } else {
      "Command failed with no error output (possibly elevation denied)"
        .to_string()
    };

    // Alright, time to error
    Err(ElevationError::CommandFailed {
      program: program.to_string(),
      code:    output.status.code().unwrap_or(-1),
      message: error_message,
    })
  }
}
