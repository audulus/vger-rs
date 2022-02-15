use crate::defs::*;
use crate::gpu_vec::*;
use crate::paint::*;
use crate::path::*;
use crate::prim::*;
use euclid::*;
use std::mem::size_of;
use wgpu::*;

pub const MAX_LAYERS: usize = 4;

pub struct Scene {
    pub prims: [GPUVec<Prim>; MAX_LAYERS],
    pub cvs: GPUVec<LocalPoint>,
    pub xforms: GPUVec<LocalToWorld>,
    pub paints: GPUVec<Paint>,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_groups: [wgpu::BindGroup; MAX_LAYERS],
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

        let cvs = GPUVec::new(device, MAX_PRIMS, "cv Buffer");
        let xforms = GPUVec::new(device, MAX_PRIMS, "Xform Buffer");
        let paints = GPUVec::new(device, MAX_PRIMS, "Paint Buffer");

        let bind_group_layout = Self::bind_group_layout(device);

        let bind_groups = [
            Scene::bind_group(device, &prims[0], &cvs, &xforms, &paints),
            Scene::bind_group(device, &prims[1], &cvs, &xforms, &paints),
            Scene::bind_group(device, &prims[2], &cvs, &xforms, &paints),
            Scene::bind_group(device, &prims[3], &cvs, &xforms, &paints),
        ];

        Self {
            prims,
            cvs,
            xforms,
            paints,
            bind_group_layout,
            bind_groups,
        }
    }

    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                GPUVec::<Prim>::bind_group_layout_entry(0),
                GPUVec::<LocalPoint>::bind_group_layout_entry(1),
                GPUVec::<LocalToWorld>::bind_group_layout_entry(2),
                GPUVec::<Paint>::bind_group_layout_entry(3),
            ],
            label: Some("bind_group_layout"),
        })
    }

    fn bind_group(
        device: &wgpu::Device,
        prims: &GPUVec<Prim>,
        cvs: &GPUVec<LocalPoint>,
        xforms: &GPUVec<LocalToWorld>,
        paints: &GPUVec<Paint>,
    ) -> wgpu::BindGroup {
        let bind_group_layout = Self::bind_group_layout(device);

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                prims.bind_group_entry(0),
                cvs.bind_group_entry(1),
                xforms.bind_group_entry(2),
                paints.bind_group_entry(3),
            ],
            label: Some("vger bind group"),
        })
    }

    pub async fn map(&self, device: &wgpu::Device) {
        for i in 0..4 {
            self.prims[i].map(device).await.unwrap();
        }
        self.cvs.map(device).await.unwrap();
        self.xforms.map(device).await.unwrap();
        self.paints.map(device).await.unwrap();
    }

    pub fn unmap(&self) {
        for i in 0..4 {
            self.prims[i].unmap();
        }
        self.cvs.unmap();
        self.xforms.unmap();
        self.paints.unmap();
    }
}
