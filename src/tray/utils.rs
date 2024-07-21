use crate::tailscale;
use crate::tray::menu::SysTray;
use std::error::Error;

type TrayServiceError = Box<dyn Error>;

pub fn start_tray_service() -> Result<(), TrayServiceError> {
    let status = tailscale::status::get_current()
        .map_err(|e| format!("Failed to update Tailscale status: {e}"))?;

    let _handle = ksni::spawn(SysTray { ctx: status })
        .map_err(|e| format!("Failed to spawn Tray implementation: {e}"))?;

    Ok(())
}
