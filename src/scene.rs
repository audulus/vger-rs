use crate::path::*;
use crate::prim::*;
use crate::gpu_vec::*;
use euclid::*;
use std::mem::size_of;
use wgpu::*;

pub struct LocalSpace {}
type LocalToWorld = Transform2D<f32, LocalSpace, WorldSpace>;

#[derive(Clone, Copy)]
pub struct Paint {
    xform: LocalToWorld,

    inner_color: [f32; 4],
    outer_color: [f32; 4],

    glow: f32,
    image: i32,
}

pub struct Scene {
    pub prim_buffer: GPUVec<Prim>,
    pub xform_buffer: GPUVec<LocalToWorld>,
    pub paint_buffer: GPUVec<Paint>,
}

const MAX_PRIMS: usize = 65536;

impl Scene {
    pub fn new(device: &wgpu::Device) -> Self {
        let prim_buffer = GPUVec::new(device, MAX_PRIMS, "Prim Buffer");
        let xform_buffer = GPUVec::new(device, MAX_PRIMS, "Xform Buffer");
        let paint_buffer = GPUVec::new(device, MAX_PRIMS, "Paint Buffer");

        Self {
            prim_buffer,
            xform_buffer,
            paint_buffer,
        }
    }
}
