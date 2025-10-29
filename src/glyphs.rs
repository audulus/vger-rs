use crate::atlas::{Atlas, AtlasContent};
use rect_packer::Rect;
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::Arc,
};

#[derive(Copy, Clone, Debug)]
pub struct GlyphInfo {
    pub rect: Option<Rect>,
    pub metrics: fontdue::Metrics,
}

#[derive(Clone, Eq)]
pub struct ImageId {
    id: Arc<bool>,
}

impl ImageId {
    pub fn new() -> Self {
        Self {
            id: Arc::new(false),
        }
    }
}

impl PartialEq for ImageId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.id, &other.id)
    }
}

impl Hash for ImageId {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        (Arc::as_ptr(&self.id) as usize).hash(state)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AtlasImage {
    pub rect: Option<Rect>,
}

pub struct AtlasCache {
    pub mask_atlas: Atlas,
    pub color_atlas: Atlas,
    pub font: fontdue::Font,
    glyphs: HashMap<(char, u32), GlyphInfo>,
    mask_images: HashMap<ImageId, AtlasImage>,
    color_images: HashMap<ImageId, AtlasImage>,
}

impl AtlasCache {
    pub fn new(device: &wgpu::Device) -> Self {
        let mut settings = fontdue::FontSettings::default();
        settings.collection_index = 0;
        settings.scale = 100.0;
        let font = include_bytes!("fonts/Anodina-Regular.ttf") as &[u8];

        Self {
            mask_atlas: Atlas::new(device, AtlasContent::Mask),
            color_atlas: Atlas::new(device, AtlasContent::Color),
            font: fontdue::Font::from_bytes(font, settings).unwrap(),
            glyphs: HashMap::new(),
            mask_images: HashMap::new(),
            color_images: HashMap::new(),
        }
    }

    pub fn get_mask_image(
        &mut self,
        id: ImageId,
        width: u32,
        height: u32,
        image: impl FnOnce() -> Vec<u8>,
    ) -> AtlasImage {
        let mask_atlas = &mut self.mask_atlas;
        *self.mask_images.entry(id).or_insert_with(|| AtlasImage {
            rect: mask_atlas.add_region(&image(), width, height),
        })
    }

    pub fn get_color_image(
        &mut self,
        id: ImageId,
        width: u32,
        height: u32,
        image: impl FnOnce() -> Vec<u8>,
    ) -> AtlasImage {
        let color_atlas = &mut self.color_atlas;
        *self.color_images.entry(id).or_insert_with(|| AtlasImage {
            rect: color_atlas.add_region(&image(), width, height),
        })
    }

    pub fn get_glyph(&mut self, c: char, size: f32) -> GlyphInfo {
        let factor = 65536.0;

        // Convert size to fixed point so we can hash it.
        let size_fixed_point = (size * factor) as u32;

        // Do we already have a glyph?
        match self.glyphs.get(&(c, size_fixed_point)) {
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

                self.glyphs.insert((c, size_fixed_point), info);
                info
            }
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        self.mask_atlas.update(device, encoder);
        self.color_atlas.update(device, encoder);
    }

    pub fn check_usage(&mut self) {
        if self.mask_atlas.usage() > 0.7 {
            self.glyphs.clear();
            self.mask_atlas.clear();
            self.mask_images.clear();
        }
        if self.color_atlas.usage() > 0.7 {
            self.color_atlas.clear();
            self.color_images.clear();
        }
    }
}
