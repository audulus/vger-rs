
use crate::atlas::Atlas;
use crate::defs::*;

struct GlyphInfo {
    size: f32,
    region_index: Option<usize>,
    texture_widht: usize,
    texture_height: usize,
    glyph_bounds: LocalRect,
}

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