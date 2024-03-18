use crate::tailscale::utils;
use crate::tray::Context;
use which::which;

pub fn get_current_status() -> Context {
    let status: utils::Status = utils::get_status().unwrap();
    let ctx = Context {
        ip: status.this_machine.ips[0].clone(),
        pkexec: which("pkexec").unwrap(),
        status,
    };

    ctx
}
