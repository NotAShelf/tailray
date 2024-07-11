use ksni::Icon;
use resvg::{
    self,
    tiny_skia::{Pixmap, Transform},
    usvg::{Options, Tree},
};

const SVG_DATA: &str = include_str!("assets/tailscale.svg");

pub struct ResvgRenderer {
    options: Options,
    transform: Transform,
    font_db: fontdb::Database,
}

impl ResvgRenderer {
    pub fn to_icon(&mut self, svg_str: &str) -> Icon {
        let rtree = Tree::from_str(svg_str, &self.options, &self.font_db).unwrap_or_else(|e| {
            panic!("Failed to parse SVG: {}", e);
        });

        let size = rtree.size();

        let mut pixmap =
            Pixmap::new(size.width().round() as u32, size.height().round() as u32).unwrap();

        resvg::render(&rtree, self.transform, &mut pixmap.as_mut());

        let argb_data: Vec<u8> = pixmap
            .take()
            .chunks(4)
            .flat_map(|rgba| [rgba[3], rgba[0], rgba[1], rgba[2]])
            .collect();

        Icon {
            width: size.width().round() as i32,
            height: size.height().round() as i32,
            data: argb_data,
        }
    }

    pub fn load_icon(enabled: bool) -> Vec<Icon> {
        let mut renderer = ResvgRenderer {
            options: Options::default(),
            transform: Transform::default(),
            font_db: fontdb::Database::new(),
        };

        match enabled {
            true => {
                log::debug!("icon: Tailscale is enabled");
                vec![renderer.to_icon(&SVG_DATA)]
            }
            false => {
                log::debug!("icon: Tailscale is not enabled");
                vec![renderer.to_icon(&SVG_DATA.replace("1.0", "0.4"))]
            }
        }
    }
}
