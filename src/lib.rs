use euclid::*;
use futures::executor::block_on;
use wgpu::*;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod path;
use path::*;
use std::mem::size_of;

mod scene;
use scene::*;

mod prim;
use prim::*;

mod gpu_vec;

pub struct VGER {
    device: wgpu::Device,
    queue: wgpu::Queue,
    scenes: [Scene; 3],
    cur_prim: [usize; MAX_LAYERS],
    cur_scene: usize,
    cur_layer: usize,
    tx_stack: Vec<LocalToWorld>
}

impl VGER {
    async fn setup() -> (wgpu::Device, wgpu::Queue) {
        let backend = wgpu::Backends::all();
        let instance = wgpu::Instance::new(backend);

        let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, backend, None)
            .await
            .expect("No suitable GPU adapters found on the system!");

        let adapter_info = adapter.get_info();
        println!("Using {} ({:?})", adapter_info.name, adapter_info.backend);

        let trace_dir = std::env::var("WGPU_TRACE");
        adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::default(),
                    limits: wgpu::Limits::default(),
                },
                trace_dir.ok().as_ref().map(std::path::Path::new),
            )
            .await
            .expect("Unable to find a suitable GPU adapter!")
    }

    pub fn new() -> Self {
        let (device, queue) = block_on(VGER::setup());

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "shader.wgsl"
            ))),
        });

        let scenes = [
            Scene::new(&device),
            Scene::new(&device),
            Scene::new(&device),
        ];

        Self {
            device,
            queue,
            scenes,
            cur_prim: [0,0,0,0],
            cur_scene: 0,
            cur_layer: 0,
            tx_stack: vec![]
        }
    }

    fn render(&mut self, prim: Prim) {

        let prim_ix = self.cur_prim[self.cur_layer];
        if prim_ix < MAX_PRIMS {
            self.scenes[self.cur_scene].prims[self.cur_layer][prim_ix] = prim;
            self.cur_prim[self.cur_layer] += 1;
        }
    
    }

    pub fn fill_circle(&mut self, center: LocalPoint, radius: f32, paint_index: usize) {

        let mut prim = Prim::default();
        prim.prim_type = PrimType::Circle;
        prim.cvs[0] = center;
        prim.radius = radius;
        prim.paint = paint_index as u32;

        self.render(prim);
    }

    pub fn stroke_arc(&mut self, center: LocalPoint, radius: f32, width: f32, rotation: f32, aperture: f32, paint_index: usize) {

        let mut prim = Prim::default();
        prim.prim_type = PrimType::Arc;
        prim.radius = radius;
        prim.cvs = [ center, LocalPoint::new(rotation.sin(), rotation.cos()), LocalPoint::new(aperture.sin(), aperture.cos()) ];
        prim.width = width;
        prim.paint = paint_index as u32;
    
        self.render(prim);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn create_vger() {
        let _ = VGER::new();
    }
}
