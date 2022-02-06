
use wgpu;
use wgpu::util::DeviceExt;
use rectangle_pack::*;
// use std::collections::BTreeMap;

struct Atlas {
    rects: GroupedRectsToPlace<u32>,
    new_data: Vec<wgpu::Buffer>
}

impl Atlas {

    pub fn add_region(&mut self, device: &wgpu::Device, data: &[u8], width: usize, height: usize) {

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