use rectangle_pack::*;
use wgpu;
use wgpu::util::DeviceExt;
// use std::collections::BTreeMap;
use rect_packer::Packer;

#[derive(Debug)]
struct ImageData {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

pub struct Atlas {
    packer: Packer,
    new_data: Vec<ImageData>,
}

impl Atlas {
    pub fn new() -> Self {
        let config = rect_packer::Config {
            width: 1024,
            height: 1024,

            border_padding: 5,
            rectangle_padding: 10,
        };

        Self {
            packer: Packer::new(config),
            new_data: vec![],
        }
    }

    pub fn add_region(&mut self, data: &[u8], width: u32, height: u32) {
        self.new_data.push(ImageData {
            width,
            height,
            data: data.into(),
        });
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        for data in &self.new_data {
            if let Some(rect) = self
                .packer
                .pack(data.width as i32, data.height as i32, false)
            {
                let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Temp Buffer"),
                    contents: &data.data,
                    usage: wgpu::BufferUsages::COPY_SRC,
                });
            }
        }

        self.new_data.clear();
    }
}
