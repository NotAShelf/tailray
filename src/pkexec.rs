use std::path::PathBuf;

use log::{debug, error};
use thiserror::Error;
use which::which;
use whoami::username;

const FALLBACK_PKEXEC_PATH: &str = "/usr/bin/pkexec";

/// Errors that can occur in `PKExec` operations
#[derive(Debug, Error)]
pub enum PkexecError {
  #[error("Failed to resolve pkexec path: {0}")]
  Resolution(#[from] which::Error),
}

/// Gets the path to the pkexec binary
///
/// Returns a Result containing either the path to pkexec or a `PkexecError`
pub fn get_path() -> Result<PathBuf, PkexecError> {
  which("pkexec")
    .inspect(|path| debug!("pkexec found at: {}", path.display()))
    .map_err(PkexecError::Resolution)
}

/// Fallback to get pkexec path or a default path if not found
///
/// This function never fails but logs warnings if pkexec can't be found
pub fn get_path_or_default() -> PathBuf {
  get_path().unwrap_or_else(|e| {
    error!("Using fallback path for pkexec: {e}");
    PathBuf::from(FALLBACK_PKEXEC_PATH)
  })
}

/// Determines if privilege elevation is needed
///
/// Returns false if the current user is root, true otherwise
pub fn should_elevate_perms() -> bool {
  let is_root = username() == "root";
  if is_root {
    debug!("Running as root, no need to elevate permissions");
  }
  !is_root
}
