use crate::gpu_vec::*;
use crate::path::*;
use crate::prim::*;
use euclid::*;
use std::mem::size_of;
use wgpu::*;

pub type LocalToWorld = Transform2D<f32, LocalSpace, WorldSpace>;

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
    pub bind_group: wgpu::BindGroup
}

pub const MAX_PRIMS: usize = 65536;

impl Scene {
    pub fn new(device: &wgpu::Device) -> Self {

        let prims = [
            GPUVec::new(device, MAX_PRIMS, "Prim Buffer 0"),
            GPUVec::new(device, MAX_PRIMS, "Prim Buffer 1"),
            GPUVec::new(device, MAX_PRIMS, "Prim Buffer 2"),
            GPUVec::new(device, MAX_PRIMS, "Prim Buffer 3"),
        ];

        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage{read_only: false},
                            has_dynamic_offset: true,
                            min_binding_size: None
                        },
                        count: None,
                    },
                ],
                label: Some("bind_group_layout"),
            }
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding{
                        buffer: prims[0].buffer(),
                        offset: 0,
                        size: None
                    }),
                },
            ],
            label: Some("vger bind group"),
        });

        Self {
            prims,
            xforms: GPUVec::new(device, MAX_PRIMS, "Xform Buffer"),
            paints: GPUVec::new(device, MAX_PRIMS, "Paint Buffer"),
            bind_group
        }
    }
}
