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

const MARK_WHITE_24: &'static str = r##"
<svg width="24" height="24" viewBox="0 0 23 23" fill="none" xmlns="http://www.w3.org/2000/svg">
<circle opacity="0.2" cx="3.4" cy="3.25" r="2.7" fill="white"></circle>
<circle cx="3.4" cy="11.3" r="2.7" fill="white"></circle>
<circle opacity="0.2" cx="3.4" cy="19.5" r="2.7" fill="white"></circle>
<circle cx="11.5" cy="11.3" r="2.7" fill="white"></circle>
<circle cx="11.5" cy="19.5" r="2.7" fill="white"></circle>
<circle opacity="0.2" cx="11.5" cy="3.25" r="2.7" fill="white"></circle>
<circle opacity="0.2" cx="19.5" cy="3.25" r="2.7" fill="white"></circle>
<circle cx="19.5" cy="11.3" r="2.7" fill="white"></circle>
<circle opacity="0.2" cx="19.5" cy="19.5" r="2.7" fill="white"></circle>
</svg>
"##;

pub fn load_icon(enabled: bool) -> Vec<ksni::Icon> {
    match enabled {
        true => vec![to_icon(MARK_WHITE_24)],
        false => vec![to_icon(&MARK_WHITE_24.replace("white", "#141414"))],
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
