
use wgpu;
use wgpu::util::DeviceExt;
use rectangle_pack::*;
// use std::collections::BTreeMap;
use rect_packer::Packer;

struct Atlas {
    packer: Packer,
    new_data: Vec<wgpu::Buffer>
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

    pub fn add_region(&mut self, device: &wgpu::Device, data: &[u8], width: u32, height: u32) {

        if let Some(rect) = self.packer.pack(width as i32, height as i32, false) {

            let buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Temp Buffer"),
                    contents: &data,
                    usage: wgpu::BufferUsages::COPY_SRC,
                }
            );

            self.new_data.push(buffer);
        }        

    }
}