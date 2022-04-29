const MARK_WHITE_24: &'static str = r##"
<svg width="32" height="32" viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg">
<defs><linearGradient id="Gradient1" x1="0%" x2="0%" y1="0%" y2="100%">
<stop offset="0%" style="stop-color:#777;stop-opacity:1" />
<stop offset="100%" style="stop-color:#222;stop-opacity:1" />
</linearGradient></defs>
<rect x="1" y="1" width="30" height="30" rx="7.7" fill="url(#Gradient1)" />
<circle opacity="0.4" cx="7.9"  cy="7.30" r="2.7" fill="white" />
<circle opacity="0.4" cx="16.0" cy="7.30" r="2.7" fill="white" />
<circle opacity="0.4" cx="24.0" cy="7.30" r="2.7" fill="white" />
<circle opacity="1.0" cx="7.9"  cy="15.8" r="2.7" fill="white" />
<circle opacity="1.0" cx="16.0" cy="15.8" r="2.7" fill="white" />
<circle opacity="1.0" cx="24.0" cy="15.8" r="2.7" fill="white" />
<circle opacity="0.4" cx="7.9"  cy="24.0" r="2.7" fill="white" />
<circle opacity="1.0" cx="16.0" cy="24.0" r="2.7" fill="white" />
<circle opacity="0.4" cx="24.0" cy="24.0" r="2.7" fill="white" />
</svg>
"##;

pub fn load_icon(enabled: bool) -> Vec<ksni::Icon> {
    match enabled {
        true => vec![to_icon(MARK_WHITE_24)],
        false => vec![to_icon(&MARK_WHITE_24.replace("1.0", "0.4"))],
    }
}
fn to_icon(svg_str: &str) -> ksni::Icon {
    let rtree = usvg::Tree::from_str(svg_str, &usvg::Options::default().to_ref()).unwrap();
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
