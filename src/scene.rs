
use wgpu::*;

struct Scene {
    pub prim_buffer: wgpu::Buffer,
    pub xform_buffer: wgpu::Buffer,
    pub paint_buffer: wgpu::Buffer
}