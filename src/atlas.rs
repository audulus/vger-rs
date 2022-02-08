use wgpu;
use wgpu::util::DeviceExt;
use rect_packer::{Packer, Rect};

#[derive(Debug)]
struct ImageData {
    rect: Rect,
    data: Vec<u8>,
}

pub struct Atlas {
    packer: Packer,
    new_data: Vec<ImageData>,
    atlas_texture: wgpu::Texture
}

impl Atlas {
    pub fn new(device: &wgpu::Device) -> Self {
        let config = rect_packer::Config {
            width: 1024,
            height: 1024,

            border_padding: 5,
            rectangle_padding: 10,
        };

        let texture_size = wgpu::Extent3d {
            width: 1024,
            height: 1024,
            depth_or_array_layers: 1,
        };

        let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::COPY_DST,
            label: Some("atlas_texture"),
        });

        Self {
            packer: Packer::new(config),
            new_data: vec![],
            atlas_texture
        }
    }

    pub fn add_region(&mut self, data: &[u8], width: u32, height: u32) -> Option<Rect> {

        if let Some(rect) = self
            .packer
            .pack(width as i32, height as i32, false) {

            self.new_data.push(ImageData {
                rect,
                data: data.into(),
            });

            Some(rect)
            
        } else {
            None
        }
        
    }

    pub fn update(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {

        for data in &self.new_data {

            // Pad data to wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
            let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as i32;
            let padding = (align - data.rect.width % align) % align;
            let padded_width = data.rect.width + padding;
            let mut padded_data = vec![];
            padded_data.reserve( (padded_width * data.rect.height) as usize );

            let mut i = 0;
            for _ in 0..data.rect.height {
                for _ in 0..data.rect.width {
                    padded_data.push(data.data[i]);
                    i += 1;
                }
                while (padded_data.len() % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize) != 0 {
                    padded_data.push(0);
                }
            }

            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Temp Buffer"),
                contents: &padded_data,
                usage: wgpu::BufferUsages::COPY_SRC,
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
                    }
                },
                wgpu::ImageCopyTexture {
                    texture: &self.atlas_texture,
                    mip_level: 0,
                    aspect: wgpu::TextureAspect::All,
                    origin: wgpu::Origin3d{x: data.rect.x as u32, y: data.rect.y as u32, z: 0},
                },
                image_size,
            );
        }

        self.new_data.clear();
    }
}
