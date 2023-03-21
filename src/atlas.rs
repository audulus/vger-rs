use rect_packer::{Packer, Rect};
use wgpu::util::DeviceExt;

#[derive(Debug)]
struct ImageData {
    rect: Rect,
    data: Vec<u8>,
}

pub enum AtlasContent {
    Mask,
    Color,
}

pub struct Atlas {
    packer: Packer,
    new_data: Vec<ImageData>,
    pub atlas_texture: wgpu::Texture,
    area_used: i32,
    did_clear: bool,
    content: AtlasContent,
}

impl Atlas {
    pub const ATLAS_SIZE: u32 = 1024;
    pub const RECT_PADDING: i32 = 6;

    fn get_packer_config() -> rect_packer::Config {
        rect_packer::Config {
            width: Atlas::ATLAS_SIZE as i32,
            height: Atlas::ATLAS_SIZE as i32,

            border_padding: Atlas::RECT_PADDING,
            rectangle_padding: Atlas::RECT_PADDING,
        }
    }

    pub fn get_texture_desc() -> wgpu::TextureDescriptor<'static> {
        let texture_size = wgpu::Extent3d {
            width: Atlas::ATLAS_SIZE,
            height: Atlas::ATLAS_SIZE,
            depth_or_array_layers: 1,
        };

        wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING,
            label: Some("atlas_texture"),
            view_formats: &[wgpu::TextureFormat::R8Unorm],
        }
    }

    pub fn new(device: &wgpu::Device, content: AtlasContent) -> Self {
        let texture_size = wgpu::Extent3d {
            width: Atlas::ATLAS_SIZE,
            height: Atlas::ATLAS_SIZE,
            depth_or_array_layers: 1,
        };
        let format = match content {
            AtlasContent::Mask => wgpu::TextureFormat::R8Unorm,
            AtlasContent::Color => wgpu::TextureFormat::Rgba8Unorm,
        };
        let desc = wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING,
            label: Some("atlas_texture"),
            view_formats: &[format],
        };
        let atlas_texture = device.create_texture(&desc);

        Self {
            packer: Packer::new(Atlas::get_packer_config()),
            new_data: vec![],
            atlas_texture,
            area_used: 0,
            did_clear: false,
            content,
        }
    }

    pub fn add_region(&mut self, data: &[u8], width: u32, height: u32) -> Option<Rect> {
        if let Some(rect) = self.packer.pack(width as i32, height as i32, false) {
            self.new_data.push(ImageData {
                rect,
                data: data.into(),
            });
            self.area_used +=
                (rect.width + Atlas::RECT_PADDING) * (rect.height + Atlas::RECT_PADDING);

            Some(rect)
        } else {
            None
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        if self.did_clear {
            // encoder.clear_texture(&self.atlas_texture, &wgpu::ImageSubresourceRange::default());

            let sz = Atlas::ATLAS_SIZE as usize;

            let data = vec![0_u8; sz * sz];

            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("atlas temp buffer"),
                contents: &data,
                usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_WRITE,
            });

            let image_size = wgpu::Extent3d {
                width: sz as u32,
                height: sz as u32,
                depth_or_array_layers: 1,
            };

            encoder.copy_buffer_to_texture(
                wgpu::ImageCopyBuffer {
                    buffer: &buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: std::num::NonZeroU32::new((sz * 4) as u32),
                        rows_per_image: None,
                    },
                },
                wgpu::ImageCopyTexture {
                    texture: &self.atlas_texture,
                    mip_level: 0,
                    aspect: wgpu::TextureAspect::All,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                },
                image_size,
            );

            self.did_clear = false;
        }

        for data in &self.new_data {
            // Pad data to wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
            let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as i32;
            let pixels = match self.content {
                AtlasContent::Mask => 1,
                AtlasContent::Color => 4,
            };
            let width = data.rect.width * pixels;
            let padding = (align - width % align) % align;
            let padded_width = width + padding;
            let mut padded_data = vec![];
            padded_data.reserve((padded_width * data.rect.height) as usize);

            let mut i = 0;
            for _ in 0..data.rect.height {
                for _ in 0..width {
                    padded_data.push(data.data[i]);
                    i += 1;
                }
                while (padded_data.len() % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize) != 0 {
                    padded_data.push(0);
                }
            }

            assert!(padded_data.len() == (padded_width * data.rect.height) as usize);

            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("atlas temp buffer"),
                contents: &padded_data,
                usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_WRITE,
            });

            let image_size = wgpu::Extent3d {
                width: data.rect.width as u32,
                height: data.rect.height as u32,
                depth_or_array_layers: 1,
            };

            encoder.copy_buffer_to_texture(
                wgpu::ImageCopyBuffer {
                    buffer: &buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: std::num::NonZeroU32::new(padded_width as u32),
                        rows_per_image: None,
                    },
                },
                wgpu::ImageCopyTexture {
                    texture: &self.atlas_texture,
                    mip_level: 0,
                    aspect: wgpu::TextureAspect::All,
                    origin: wgpu::Origin3d {
                        x: data.rect.x as u32,
                        y: data.rect.y as u32,
                        z: 0,
                    },
                },
                image_size,
            );
        }

        self.new_data.clear();
    }

    pub fn create_view(&self) -> wgpu::TextureView {
        self.atlas_texture
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn usage(&self) -> f32 {
        (self.area_used as f32) / ((Atlas::ATLAS_SIZE * Atlas::ATLAS_SIZE) as f32)
    }

    pub fn clear(&mut self) {
        self.packer = Packer::new(Atlas::get_packer_config());
        self.area_used = 0;
        self.new_data.clear();
        self.did_clear = true;
    }
}
