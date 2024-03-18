use std::path::PathBuf;
use which::which;

pub fn get_pkexec_path() -> PathBuf {
    match which("pkexec") {
        Ok(path) => path,
        Err(_) => panic!("pkexec not found in PATH"),
    }
}

pub fn pkexec_found(pkexec_path: &PathBuf) -> bool {
    pkexec_path.is_file()
}
