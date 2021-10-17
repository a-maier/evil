use lazy_static::lazy_static;

#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub(crate) struct Image {
    size: (usize, usize),
    srgba_pixels: Vec<egui::Color32>,
    svg: String
}

lazy_static!{
    static ref OPT: usvg::Options = {
        let mut opt = usvg::Options::default();
        opt.fontdb.load_system_fonts();
        opt
    };
}

impl Image {
    pub fn new(svg: String, size: (usize, usize)) -> Self {
        let tree = usvg::Tree::from_str(&svg, &OPT.to_ref()).unwrap();
        let fit = usvg::FitTo::Size(size.0 as u32, size.1 as u32);
        let mut pixmap = tiny_skia::Pixmap::new(size.0 as u32, size.1 as u32).unwrap();
        resvg::render(&tree, fit, pixmap.as_mut());

        let srgba_pixels = pixmap.data().chunks(4).map(
            |rgba| egui::Color32::from_rgba_premultiplied(rgba[0], rgba[1], rgba[2], rgba[3])
        ).collect();
        Image {
            svg,
            srgba_pixels,
            size,
        }
    }

    pub fn pixels(&self) -> &[egui::Color32] {
        &self.srgba_pixels
    }

    pub fn size(&self) -> (usize, usize) {
        self.size
    }
}
