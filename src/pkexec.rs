use std::path::PathBuf;
use which::which;
use whoami::username;

pub fn get_path() -> PathBuf {
    which("pkexec").unwrap_or_else(|_| panic!("pkexec not found in PATH"))
}

// We don't need to elevate privileges if we're using the Tray service
// as the root user. This shouldn't really happen, but it's possible
// depending on how Tailran is ran.
pub fn should_elevate_perms() -> bool {
    let parent_user = username();

    if parent_user.eq("root") {
        return false;
    }

    true
}
