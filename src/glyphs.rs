
use crate::atlas::Atlas;
use crate::defs::*;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug)]
struct GlyphInfo {
    size: u32,
    region_index: Option<usize>,
    metrics: fontdue::Metrics,
}

struct GlyphCache {
    atlas: Atlas,
    font: fontdue::Font,
    info: HashMap<(char, u32), GlyphInfo>
}

impl GlyphCache {
    pub fn new(device: &wgpu::Device) -> Self {
        
        let font = include_bytes!("fonts/Anodina-Regular.ttf") as &[u8];

        Self {
            atlas: Atlas::new(device),
            font: fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap(),
            info: HashMap::new(),
        }
    }

    pub fn get_glyph(&mut self, c: char, size: u32) -> GlyphInfo {

        // Do we already have a glyph?
        match self.info.get( &(c, size) ) {
            Some(info) => *info,
            None => {
                let (metrics, data) = self.font.rasterize(c, size as f32);

                self.atlas.add_region(&data, metrics.width as u32, metrics.height as u32);

                let info = GlyphInfo {
                    size,
                    region_index: None,
                    metrics
                };

                self.info.insert( (c, size), info);
                info
            }
        }
    }
}