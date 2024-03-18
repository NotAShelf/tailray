use resvg::{
    self,
    tiny_skia::{Pixmap, Transform},
    usvg::{Options, Tree},
};

use ksni::Icon;

pub struct ResvgRenderer {
    options: Options,
    transform: Transform,
    font_db: fontdb::Database,
}

const SVG_DATA: &str = include_str!("assets/tailscale.svg");

impl ResvgRenderer {
    pub fn to_icon(&mut self, svg_str: &str) -> Icon {
        let rtree = Tree::from_str(svg_str, &self.options, &self.font_db).unwrap_or_else(|e| {
            panic!("Failed to parse SVG: {}", e);
        });

        let size = rtree.size();

        let mut pixmap =
            Pixmap::new(size.width().round() as u32, size.height().round() as u32).unwrap();

        resvg::render(&rtree, self.transform, &mut pixmap.as_mut());

        Icon {
            width: pixmap.width() as i32,
            height: pixmap.height() as i32,
            data: pixmap.take(),
        }
    }

    pub fn load_icon(enabled: bool) -> Vec<Icon> {
        let mut renderer = ResvgRenderer {
            options: Options::default(),
            transform: Transform::default(),
            font_db: fontdb::Database::new(),
        };

        match enabled {
            true => vec![renderer.to_icon(&SVG_DATA)],
            false => vec![renderer.to_icon(&SVG_DATA.replace("1.0", "0.4"))],
        }
    }
}
