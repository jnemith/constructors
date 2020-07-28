use wgpu_glyph::{ab_glyph::FontArc, GlyphBrush, GlyphBrushBuilder};

pub struct Txt {
    pub debug_text: String,
    pub glyph_brush: GlyphBrush<()>,
}

impl Txt {
    pub fn new(debug_text: String, device: &wgpu::Device) -> Self {
        let font = FontArc::try_from_slice(include_bytes!("../munro.ttf")).expect("Font not found");

        let glyph_brush =
            GlyphBrushBuilder::using_font(font).build(&device, wgpu::TextureFormat::Bgra8Unorm);

        Self {
            debug_text,
            glyph_brush,
        }
    }

    pub fn update_debug(&mut self, player: &crate::player::Player) {
        let new_text = format!(
            "x: {:.3}, y: {:.3}, z: {:.3}",
            player.camera.position.x, player.camera.position.y, player.camera.position.z
        );
        self.debug_text = new_text;
    }
}
