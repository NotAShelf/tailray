use ksni::Icon;

pub(crate) fn load(enabled: bool) -> Vec<Icon> {
    let args = match enabled {
        true => "assets/Tailscale-Mark-Black.svg",
        false => "assets/Tailscale-Mark-White.svg",
    };

    let mut opt = usvg::Options::default();
    // Get file's absolute directory.
    opt.resources_dir = std::fs::canonicalize(args).ok().and_then(|p| p.parent().map(|p| p.to_path_buf()));
    // opt.fontdb.load_system_fonts();

    let svg_data = std::fs::read(args).unwrap();
    let rtree = usvg::Tree::from_data(&svg_data, &opt.to_ref()).unwrap();

    let pixmap_size = rtree.svg_node().size.to_screen_size();
    // let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(&rtree, usvg::FitTo::Original, tiny_skia::Transform::default(), pixmap.as_mut()).unwrap();
    // pixmap.save_png("icon.png").unwrap();
    vec![Icon {
        data: pixmap.take(),
        width: pixmap_size.width() as i32,
        height: pixmap_size.height() as i32,
    }]
}
