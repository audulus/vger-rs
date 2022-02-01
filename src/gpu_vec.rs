use mem_align::MemAlign;
use wgpu::*;

pub struct GPUVec<T: Copy> {
    buffer: wgpu::Buffer,
    mem_align: MemAlign<T>,
}

impl<T: Copy> GPUVec<T> {
    pub fn new(device: &wgpu::Device, capacity: usize, label: &str) -> Self {
        let mem_align = MemAlign::new(capacity);

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: mem_align.byte_size() as _,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: true,
        });

        Self { buffer, mem_align }
    }

    pub fn new_uniforms(device: &wgpu::Device, label: &str) -> Self {
        let mem_align = MemAlign::new(1);

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: mem_align.byte_size() as _,
            usage: BufferUsages::UNIFORM,
            mapped_at_creation: true,
        });

        Self { buffer, mem_align }
    }

    pub fn capacity(&self) -> usize {
        self.mem_align.capacity()
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding: binding,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &self.buffer,
                offset: 0,
                size: None,
            }),
        }
    }
}

impl<T: Copy> std::ops::Index<usize> for GPUVec<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        let view = self.buffer.slice(..).get_mapped_range();
        let slice =
            unsafe { std::slice::from_raw_parts(view.as_ptr() as *const T, self.capacity()) };
        &slice[index]
    }
}

impl<T: Copy> std::ops::IndexMut<usize> for GPUVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let view = self.buffer.slice(..).get_mapped_range();
        let slice =
            unsafe { std::slice::from_raw_parts_mut(view.as_ptr() as *mut T, self.capacity()) };
        &mut slice[index]
    }
}
