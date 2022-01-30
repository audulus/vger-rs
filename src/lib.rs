
use wgpu::*;
use futures::executor::block_on;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use euclid::*;

mod path;
use path::*;
use std::mem::size_of;

mod scene;
use scene::*;

struct LocalSpace {}
type LocalToWorld = Transform2D<f32, LocalSpace, WorldSpace>;

pub struct VGER {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub prim_buffer: wgpu::Buffer,
    pub xform_buffer: wgpu::Buffer,
    pub paint_buffer: wgpu::Buffer
}

const MAX_PRIMS: usize = 65536;

struct Prim {

    /// Type of primitive.
    prim_type: u32,

    /// Stroke width.
    width: f32,

    /// Radius of circles. Corner radius for rounded rectangles.
    radius: f32,

    /// Control vertices.
    cvs: [f32; 6],

    /// Start of the control vertices, if they're in a separate buffer.
    start: u32,

    /// Number of control vertices (vgerCurve and vgerPathFill)
    count: u32,

    /// Index of paint applied to drawing region.
    paint: u32,

    /// Glyph region index. (used internally)
    glyph: u32,

    /// Index of transform applied to drawing region. (used internally)
    xform: u32,

    /// Min and max coordinates of the quad we're rendering. (used internally)
    quad_bounds: [f32; 4],

    /// Min and max coordinates in texture space. (used internally)
    tex_bounds: [f32; 4]

}

struct Paint {

    xform: LocalToWorld,

    inner_color: [f32; 4],
    outer_color: [f32; 4],

    glow: f32,
    image: i32,

}

impl VGER {

    async fn setup() -> (wgpu::Device, wgpu::Queue) {

        let backend = wgpu::Backends::all();
        let instance = wgpu::Instance::new(backend);

        let adapter =
        wgpu::util::initialize_adapter_from_env_or_default(&instance, backend, None)
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
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let prim_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Prim Buffer"),
                size: (MAX_PRIMS * size_of::<Prim>()) as u64,
                usage: BufferUsages::MAP_WRITE,
                mapped_at_creation: true
            }
        );

        let xform_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Xform Buffer"),
                size: (MAX_PRIMS * size_of::<LocalToWorld>()) as u64,
                usage: BufferUsages::MAP_WRITE,
                mapped_at_creation: true
            }
        );

        let paint_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Paint Buffer"),
                size: (MAX_PRIMS * size_of::<Paint>()) as u64,
                usage: BufferUsages::MAP_WRITE,
                mapped_at_creation: true
            }
        );

        Self { device, queue, prim_buffer, xform_buffer, paint_buffer }
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
