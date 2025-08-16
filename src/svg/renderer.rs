use std::{error::Error, fmt};

use ksni::Icon;
use log::{debug, error};
use resvg::{
    self,
    tiny_skia::{Pixmap, Transform},
    usvg::{Options, Tree},
};

const SVG_DATA: &str = include_str!("assets/tailscale.svg");

const DISABLED_OPACITY: &str = "0.4";
const ENABLED_OPACITY: &str = "1.0";

#[derive(Debug)]
pub enum RenderError {
    TreeParsing(String),
    PixmapCreation(String),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TreeParsing(msg) => write!(f, "Failed to parse SVG: {msg}"),
            Self::PixmapCreation(msg) => write!(f, "Failed to create pixmap: {msg}"),
        }
    }
}

impl Error for RenderError {}

/// SVG renderer for tailscale icons
#[derive(Debug, Default)]
pub struct Resvg<'a> {
    options: Options<'a>,
    transform: Transform,
}

impl Resvg<'_> {
    /// Convert an SVG string to a KDE Systray Icon
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    pub fn to_icon(&self, svg_str: &str) -> Result<Icon, RenderError> {
        // Parse the SVG string
        let tree = Tree::from_str(svg_str, &self.options)
            .map_err(|e| RenderError::TreeParsing(e.to_string()))?;

        // Get the size from the SVG
        let size = tree.size();
        let width = size.width() as u32;
        let height = size.height() as u32;

        // Create a pixmap to render into
        let mut pixmap = Pixmap::new(width, height)
            .ok_or_else(|| RenderError::PixmapCreation("Failed to create pixmap".into()))?;

        // Render the SVG to the pixmap
        resvg::render(&tree, self.transform, &mut pixmap.as_mut());

        // Convert from RGBA to ARGB format for KDE system tray
        let argb_data: Vec<u8> = pixmap
            .take()
            .chunks_exact(4)
            .flat_map(|rgba| [rgba[3], rgba[0], rgba[1], rgba[2]])
            .collect();

        // Create the Icon
        Ok(Icon {
            width: size.width() as i32,
            height: size.height() as i32,
            data: argb_data,
        })
    }

    /// Load appropriate icon based on connection state
    pub fn load_icon(enabled: bool) -> Vec<Icon> {
        let renderer = Self::default();

        if enabled {
            debug!("Loading enabled Tailscale icon");
            match renderer.to_icon(SVG_DATA) {
                Ok(icon) => vec![icon],
                Err(e) => {
                    error!("Failed to load enabled icon: {e}");
                    Vec::new()
                }
            }
        } else {
            debug!("Loading disabled Tailscale icon");
            // Replace opacity in SVG
            let disabled_svg = SVG_DATA.replace(ENABLED_OPACITY, DISABLED_OPACITY);
            match renderer.to_icon(&disabled_svg) {
                Ok(icon) => vec![icon],
                Err(e) => {
                    error!("Failed to load disabled icon: {e}");
                    Vec::new()
                }
            }
        }
    }
}
