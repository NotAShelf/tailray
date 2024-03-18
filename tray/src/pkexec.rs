use which::which;

// get the path of pkexec command and return it as a string
// in a shape that can be used in static str or constants
pub fn get_pkexec_path() -> String {
    which("pkexec").unwrap().to_str().unwrap().to_string()
}
