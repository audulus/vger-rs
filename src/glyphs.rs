
use crate::atlas::Atlas;

struct GlyphCache {
    atlas: Atlas,
    font: fontdue::Font
}

impl GlyphCache {
    pub fn new(device: &wgpu::Device) -> Self {
        
        let font = include_bytes!("fonts/Anodina-Regular.ttf") as &[u8];

        Self {
            atlas: Atlas::new(device),
            font: fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap()
        }
    }
}