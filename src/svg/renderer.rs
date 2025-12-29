use std::{error::Error, fmt};

use ksni::Icon;
use log::{debug, error};
use resvg::{
  self,
  tiny_skia::{Pixmap, Transform},
  usvg::{Options, Tree},
};

const SVG_DATA_LIGHT: &str = include_str!("assets/tailscale-light.svg");
const SVG_DATA_DARK: &str = include_str!("assets/tailscale-dark.svg");

const DISABLED_OPACITY: &str = "0.4";
const ENABLED_OPACITY: &str = "1.0";

/// Icon theme variant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
  #[default]
  Light,
  Dark,
}

impl Theme {
  /// Parse theme from environment variable TAILRAY_THEME
  pub fn from_env() -> Self {
    std::env::var("TAILRAY_THEME")
      .map(|s| {
        let s = s.to_lowercase();
        match s.as_str() {
          "dark" => Theme::Dark,
          "light" => Theme::Light,
          _ => {
            log::warn!("Invalid theme value '{s}', defaulting to light");
            Theme::Light
          },
        }
      })
      .unwrap_or_default()
  }

  /// Get the SVG data for this theme
  const fn svg_data(&self) -> &'static str {
    match self {
      Self::Light => SVG_DATA_LIGHT,
      Self::Dark => SVG_DATA_DARK,
    }
  }
}

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
  options:   Options<'a>,
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
    let mut pixmap = Pixmap::new(width, height).ok_or_else(|| {
      RenderError::PixmapCreation("Failed to create pixmap".into())
    })?;

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
      width:  size.width() as i32,
      height: size.height() as i32,
      data:   argb_data,
    })
  }

  /// Load appropriate icon based on connection state and theme
  pub fn load_icon(theme: Theme, enabled: bool) -> Vec<Icon> {
    let renderer = Self::default();
    let svg_data = theme.svg_data();

    if enabled {
      debug!("Loading enabled Tailscale icon (theme: {theme:?})");
      match renderer.to_icon(svg_data) {
        Ok(icon) => vec![icon],
        Err(e) => {
          error!("Failed to load enabled icon: {e}");
          Vec::new()
        },
      }
    } else {
      debug!("Loading disabled Tailscale icon (theme: {theme:?})");
      // Replace opacity in SVG
      let disabled_svg = svg_data.replace(ENABLED_OPACITY, DISABLED_OPACITY);
      match renderer.to_icon(&disabled_svg) {
        Ok(icon) => vec![icon],
        Err(e) => {
          error!("Failed to load disabled icon: {e}");
          Vec::new()
        },
      }
    }
  }
}
