const MARK_WHITE: &'static str = r##"
<svg width="120" height="120" viewBox="0 0 120 120" fill="none" xmlns="http://www.w3.org/2000/svg">
<circle cx="40.625" cy="59.5" r="6.625" fill="white"/>
<circle cx="60.4999" cy="59.5" r="6.625" fill="white"/>
<circle opacity="0.2" cx="40.625" cy="79.375" r="6.625" fill="white"/>
<circle opacity="0.2" cx="80.375" cy="79.375" r="6.625" fill="white"/>
<circle cx="60.4999" cy="79.375" r="6.625" fill="white"/>
<circle cx="80.375" cy="59.5" r="6.625" fill="white"/>
<circle opacity="0.2" cx="40.625" cy="39.625" r="6.625" fill="white"/>
<circle opacity="0.2" cx="60.4999" cy="39.625" r="6.625" fill="white"/>
<circle opacity="0.2" cx="80.375" cy="39.625" r="6.625" fill="white"/>
</svg>
"##;

const MARK_BLACK: &'static str = r##"
<svg width="120" height="120" viewBox="0 0 120 120" fill="none" xmlns="http://www.w3.org/2000/svg">
<circle cx="40.625" cy="59.5" r="6.625" fill="#141414"/>
<circle cx="60.4999" cy="59.5" r="6.625" fill="#141414"/>
<circle opacity="0.2" cx="40.625" cy="79.375" r="6.625" fill="#141414"/>
<circle opacity="0.2" cx="80.375" cy="79.375" r="6.625" fill="#141414"/>
<circle cx="60.4999" cy="79.375" r="6.625" fill="#141414"/>
<circle cx="80.375" cy="59.5" r="6.625" fill="#141414"/>
<circle opacity="0.2" cx="40.625" cy="39.625" r="6.625" fill="#141414"/>
<circle opacity="0.2" cx="60.4999" cy="39.625" r="6.625" fill="#141414"/>
<circle opacity="0.2" cx="80.375" cy="39.625" r="6.625" fill="#141414"/>
</svg>
"##;

pub fn load_icon(enabled: bool) -> Vec<ksni::Icon> {
    match enabled {
        true => vec![to_icon(MARK_WHITE)],
        false => vec![to_icon(MARK_BLACK)],
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
