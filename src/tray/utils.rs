use crate::tailscale;
use crate::tray::menu::SysTray;

pub fn start_tray_service() {
    // start the tray service
    let _handle = ksni::spawn(SysTray {
        ctx: tailscale::status::get_current_status(),
    })
    .unwrap_or_else(|e| {
        panic!("Failed to start the tray service: {}", e);
    });
}
