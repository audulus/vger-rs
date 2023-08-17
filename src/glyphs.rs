use crate::atlas::{Atlas, AtlasContent};
use cosmic_text::{SubpixelBin, SwashContent, SwashImage};
use rect_packer::Rect;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug)]
pub struct GlyphInfo {
    pub rect: Option<Rect>,
    pub metrics: fontdue::Metrics,
}

#[derive(Copy, Clone, Debug)]
pub struct AtlasInfo {
    pub rect: Option<Rect>,
    pub left: i32,
    pub top: i32,
    pub colored: bool,
}

pub struct GlyphCache {
    pub mask_atlas: Atlas,
    pub color_atlas: Atlas,
    pub font: fontdue::Font,
    info: HashMap<(char, u32), GlyphInfo>,
    atlas_infos: HashMap<(cosmic_text::fontdb::ID, u16, u32, SubpixelBin), AtlasInfo>,
    svg_infos: HashMap<Vec<u8>, HashMap<(u32, u32), AtlasInfo>>,
}

impl GlyphCache {
    pub fn new(device: &wgpu::Device) -> Self {
        let settings = fontdue::FontSettings {
            collection_index: 0,
            scale: 100.0,
        };
        let font = include_bytes!("fonts/Anodina-Regular.ttf") as &[u8];

        Self {
            mask_atlas: Atlas::new(device, AtlasContent::Mask),
            color_atlas: Atlas::new(device, AtlasContent::Color),
            font: fontdue::Font::from_bytes(font, settings).unwrap(),
            info: HashMap::new(),
            atlas_infos: HashMap::new(),
            svg_infos: HashMap::new(),
        }
    }

    pub fn get_svg_mask(
        &mut self,
        hash: &[u8],
        width: u32,
        height: u32,
        image: impl FnOnce() -> Vec<u8>,
    ) -> AtlasInfo {
        if !self.svg_infos.contains_key(hash) {
            self.svg_infos.insert(hash.to_vec(), HashMap::new());
        }

        {
            let svg_infos = self.svg_infos.get(hash).unwrap();
            if let Some(info) = svg_infos.get(&(width, height)) {
                return info.clone();
            }
        }

        let data = image();
        let rect = self.color_atlas.add_region(&data, width, height);
        let info = AtlasInfo {
            rect,
            left: 0,
            top: 0,
            colored: true,
        };

        let svg_infos = self.svg_infos.get_mut(hash).unwrap();
        svg_infos.insert((width, height), info.clone());

        info
    }

    pub fn get_glyph_mask<'a>(
        &mut self,
        font_id: cosmic_text::fontdb::ID,
        glyph_id: u16,
        size: u32,
        subpx: SubpixelBin,
        image: impl FnOnce() -> SwashImage,
    ) -> AtlasInfo {
        let key = (font_id, glyph_id, size, subpx);
        if let Some(rect) = self.atlas_infos.get(&key) {
            return *rect;
        }

        let image = image();
        let rect = match image.content {
            SwashContent::Mask => self.mask_atlas.add_region(
                &image.data,
                image.placement.width,
                image.placement.height,
            ),
            SwashContent::SubpixelMask | SwashContent::Color => self.color_atlas.add_region(
                &image.data,
                image.placement.width,
                image.placement.height,
            ),
        };
        let info = AtlasInfo {
            rect,
            left: image.placement.left,
            top: image.placement.top,
            colored: image.content != SwashContent::Mask,
        };
        self.atlas_infos.insert(key, info);
        info
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
                    self.mask_atlas
                        .add_region(&data, metrics.width as u32, metrics.height as u32);

                let info = GlyphInfo { rect, metrics };

                self.info.insert((c, size_fixed_point), info);
                info
            }
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        self.mask_atlas.update(device, encoder);
        self.color_atlas.update(device, encoder);
    }

    pub fn check_usage(&mut self) {
        if self.mask_atlas.usage() > 0.7 || self.color_atlas.usage() > 0.7 {
            self.clear();
        }
    }

    pub fn clear(&mut self) {
        self.info.clear();
        self.mask_atlas.clear();
        self.color_atlas.clear();
        self.atlas_infos.clear();
    }
}
