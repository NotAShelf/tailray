use std::error::Error;
use std::fmt;

use crate::pkexec::PkexecError;
use crate::svg::renderer::RenderError;
use crate::tailscale::peer::PeerError;
use crate::tailscale::status::StatusError;
use crate::tray::menu::TrayError;

#[derive(Debug)]
pub enum AppError {
    // Subsystem errors
    Pkexec(PkexecError),
    Render(RenderError),
    Peer(PeerError),
    Status(StatusError),
    Tray(TrayError),

    // External library errors
    Clipboard(arboard::Error),
    Io(std::io::Error),
    SerdeJson(serde_json::Error),
    Notify(notify_rust::error::Error),

    // Generic string error
    Message(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Pkexec(e) => write!(f, "Pkexec error: {e}"),
            AppError::Render(e) => write!(f, "SVG render error: {e}"),
            AppError::Peer(e) => write!(f, "Peer error: {e}"),
            AppError::Status(e) => write!(f, "Tailscale status error: {e}"),
            AppError::Tray(e) => write!(f, "Tray error: {e}"),
            AppError::Clipboard(e) => write!(f, "Clipboard error: {e}"),
            AppError::Io(e) => write!(f, "IO error: {e}"),
            AppError::SerdeJson(e) => write!(f, "JSON error: {e}"),
            AppError::Notify(e) => write!(f, "Notification error: {e}"),
            AppError::Message(msg) => write!(f, "{msg}"),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::Pkexec(e) => Some(e),
            AppError::Render(e) => Some(e),
            AppError::Peer(e) => Some(e),
            AppError::Status(e) => Some(e),
            AppError::Tray(e) => Some(e),
            AppError::Clipboard(e) => Some(e),
            AppError::Io(e) => Some(e),
            AppError::SerdeJson(e) => Some(e),
            AppError::Notify(e) => Some(e),
            AppError::Message(_) => None,
        }
    }
}

// From conversions for subsystem errors
impl From<PkexecError> for AppError {
    fn from(e: PkexecError) -> Self {
        AppError::Pkexec(e)
    }
}
impl From<RenderError> for AppError {
    fn from(e: RenderError) -> Self {
        AppError::Render(e)
    }
}
impl From<PeerError> for AppError {
    fn from(e: PeerError) -> Self {
        AppError::Peer(e)
    }
}
impl From<StatusError> for AppError {
    fn from(e: StatusError) -> Self {
        AppError::Status(e)
    }
}
impl From<TrayError> for AppError {
    fn from(e: TrayError) -> Self {
        AppError::Tray(e)
    }
}

// From conversions for external errors
impl From<arboard::Error> for AppError {
    fn from(e: arboard::Error) -> Self {
        AppError::Clipboard(e)
    }
}
impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}
impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::SerdeJson(e)
    }
}
impl From<notify_rust::error::Error> for AppError {
    fn from(e: notify_rust::error::Error) -> Self {
        AppError::Notify(e)
    }
}

// From conversion for string messages
impl From<String> for AppError {
    fn from(e: String) -> Self {
        AppError::Message(e)
    }
}
impl From<&str> for AppError {
    fn from(e: &str) -> Self {
        AppError::Message(e.to_owned())
    }
}

// From conversion for Box<dyn Error>
impl From<Box<dyn std::error::Error>> for AppError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        AppError::Message(e.to_string())
    }
}

// From conversion for ksni::Error
impl From<ksni::Error> for AppError {
    fn from(e: ksni::Error) -> Self {
        AppError::Message(e.to_string())
    }
}
