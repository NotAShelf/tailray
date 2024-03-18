use std::env;
use std::fs;
use usvg::Tree;

pub fn load_icon(enabled: bool) -> Vec<ksni::Icon> {
    let crate_path = env::var("CARGO_MANIFEST_DIR").unwrap();
    let svg_path = format!("{}/src/svg/assets/tailscale.svg", crate_path);
    let svg_data = fs::read_to_string(svg_path).expect("Failed to read SVG file");
    match enabled {
        true => vec![to_icon(&svg_data)],
        false => vec![to_icon(&svg_data.replace("1.0", "0.4"))],
    }
}

pub fn to_icon(svg_str: &str) -> ksni::Icon {
    let rtree = Tree::from_str(svg_str, &usvg::Options::default().to_ref()).unwrap();
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

    ksni::Icon {
        width: pixmap.width() as i32,
        height: pixmap.height() as i32,
        data: pixmap.take(),
    }
}
