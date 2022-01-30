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

pub const MAX_LAYERS: usize = 4;

pub struct Scene {
    pub prims: [GPUVec<Prim>; MAX_LAYERS],
    pub xforms: GPUVec<LocalToWorld>,
    pub paints: GPUVec<Paint>,
}

const MAX_PRIMS: usize = 65536;

impl Scene {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            prims: [
                GPUVec::new(device, MAX_PRIMS, "Prim Buffer"),
                GPUVec::new(device, MAX_PRIMS, "Prim Buffer"),
                GPUVec::new(device, MAX_PRIMS, "Prim Buffer"),
                GPUVec::new(device, MAX_PRIMS, "Prim Buffer")
            ],
            xforms: GPUVec::new(device, MAX_PRIMS, "Xform Buffer"),
            paints: GPUVec::new(device, MAX_PRIMS, "Paint Buffer"),
        }
    }
}
