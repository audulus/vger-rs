use crate::path::*;
use crate::prim::*;
use crate::gpu_vec::*;
use euclid::*;
use std::mem::size_of;
use wgpu::*;

pub struct Scene {
    pub prim_buffer: GPUVec<Prim>,
    pub xform_buffer: wgpu::Buffer,
    pub paint_buffer: wgpu::Buffer,
}

const MAX_PRIMS: usize = 65536;

struct LocalSpace {}
type LocalToWorld = Transform2D<f32, LocalSpace, WorldSpace>;

struct Paint {
    xform: LocalToWorld,

    inner_color: [f32; 4],
    outer_color: [f32; 4],

    glow: f32,
    image: i32,
}

impl Scene {
    pub fn new(device: &wgpu::Device) -> Self {
        let prim_buffer = GPUVec::new(device, MAX_PRIMS, "Prim Buffer");

        let xform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Xform Buffer"),
            size: (MAX_PRIMS * size_of::<LocalToWorld>()) as u64,
            usage: BufferUsages::MAP_WRITE,
            mapped_at_creation: true,
        });

        let paint_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Paint Buffer"),
            size: (MAX_PRIMS * size_of::<Paint>()) as u64,
            usage: BufferUsages::MAP_WRITE,
            mapped_at_creation: true,
        });

        Self {
            prim_buffer,
            xform_buffer,
            paint_buffer,
        }
    }
}
