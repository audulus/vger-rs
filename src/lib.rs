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

mod gpu_vec;

pub struct VGER {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub scenes: [Scene; 3],
    pub cur_prim: [usize; MAX_LAYERS],
    pub cur_layer: usize
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

        let cur_prim = [0,0,0,0];

        let cur_layer = 0;

        Self {
            device,
            queue,
            scenes,
            cur_prim,
            cur_layer
        }
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
