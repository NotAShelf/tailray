use std::fs::{canonicalize, read};

use ksni::Icon;
use usvg::{Options, OptionsRef, Tree};

pub(crate) fn load() -> Vec<Icon> {
    let icons = vec![
        "tray/assets/Tailscale-Mark-Black.svg",
        "tray/assets/Tailscale-Mark-White.svg",
    ];

    let mut opt = Options::default();
    // Get file's absolute directory.
    opt.resources_dir = canonicalize(icons[0])
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
    let pixmap1 = load_svg(icons[0], &opt.to_ref());
    let pixmap2 = load_svg(icons[1], &opt.to_ref());
    vec![pixmap1, pixmap2]
}
fn load_svg(path: &str, opt: &OptionsRef) -> Icon {
    // fn load_svg(path: &str, opt: &OptionsRef) -> Pixmap {
    let data = read(path).unwrap();
    let rtree = Tree::from_data(&data, opt).unwrap();
    let pixmap_size = rtree.svg_node().size;
    let mut pixmap = tiny_skia::Pixmap::new(
        pixmap_size.width().round() as u32,
        pixmap_size.height().round() as u32,
    )
    .unwrap();
    resvg::render(
        &rtree,
        usvg::FitTo::Original,
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .unwrap();

    Icon {
        width: pixmap.width() as i32,
        height: pixmap.height() as i32,
        data: pixmap.take(),
    }
}
