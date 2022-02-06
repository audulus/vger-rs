
use wgpu;
use wgpu::util::DeviceExt;
use rectangle_pack::*;
// use std::collections::BTreeMap;

struct Atlas {
    rects: GroupedRectsToPlace<u32>,
    new_data: Vec<wgpu::Buffer>
}

impl Atlas {

    pub fn add_region(&mut self, device: &wgpu::Device, data: &[u8], width: u32, height: u32) {

        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Temp Buffer"),
                contents: &data,
                usage: wgpu::BufferUsages::COPY_SRC,
            }
        );

        self.rects.push_rect(
            self.new_data.len() as u32,
            None,
            RectToInsert::new(width, height, 0)
        );

        self.new_data.push(buffer);

    }
}