use crate::atlas::Atlas;
use rect_packer::Rect;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug)]
pub struct GlyphInfo {
    pub rect: Option<Rect>,
    pub metrics: fontdue::Metrics,
}

pub struct GlyphCache {
    atlas: Atlas,
    pub font: fontdue::Font,
    info: HashMap<(char, u32), GlyphInfo>,
}

impl GlyphCache {
    pub fn new(device: &wgpu::Device) -> Self {
        let settings = fontdue::FontSettings {
            collection_index: 0,
            scale: 100.0,
        };
        let font = include_bytes!("fonts/Anodina-Regular.ttf") as &[u8];

        Self {
            atlas: Atlas::new(device),
            font: fontdue::Font::from_bytes(font, settings).unwrap(),
            info: HashMap::new(),
        }
    }

    pub fn get_glyph(&mut self, c: char, size: f32) -> GlyphInfo {
        let factor = 65536.0;

        // Convert size to fixed point so we can hash it.
        let size_fixed_point = (size * factor) as u32;

        // Do we already have a glyph?
        match self.info.get(&(c, size_fixed_point)) {
            Some(info) => *info,
            None => {
                let (metrics, data) = self.font.rasterize(c, size_fixed_point as f32 / factor);

                /*
                let mut i = 0;
                for _ in 0..metrics.height {
                    for _ in 0..metrics.width {
                        print!("{} ", if data[i] != 0 { '*' } else { ' ' });
                        i += 1;
                    }
                    print!("\n");
                }
                */

                let rect =
                    self.atlas
                        .add_region(&data, metrics.width as u32, metrics.height as u32);

                let info = GlyphInfo { rect, metrics };

                self.info.insert((c, size_fixed_point), info);
                info
            }
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        self.atlas.update(device, encoder);
    }

    pub fn create_view(&self) -> wgpu::TextureView {
        self.atlas.create_view()
    }

    pub fn usage(&self) -> f32 {
        self.atlas.usage()
    }

    pub fn clear(&mut self) {
        self.info.clear();
        self.atlas.clear();
    }
}
