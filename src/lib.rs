use euclid::*;
// use wgpu::*;

/*
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
*/

mod path;
// use path::*;

mod scene;
use scene::*;

mod prim;
use prim::*;

mod defs;
use defs::*;

mod paint;
use paint::*;

mod gpu_vec;
use gpu_vec::*;

mod color;
use color::Color;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
struct Uniforms {
    size: [f32; 2],
}

pub struct PaintIndex {
    index: usize,
}

pub struct VGER {
    scenes: [Scene; 3],
    cur_prim: [usize; MAX_LAYERS],
    cur_scene: usize,
    cur_layer: usize,
    tx_stack: Vec<LocalToWorld>,
    device_px_ratio: f32,
    screen_size: ScreenSize,
    paint_count: usize,
    pipeline: wgpu::RenderPipeline,
    uniform_bind_group: wgpu::BindGroup,
    uniforms: GPUVec<Uniforms>,
    xform_count: usize,
}

impl VGER {
    pub fn new(device: &wgpu::Device) -> Self {
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

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let uniforms = GPUVec::new_uniforms(device, "uniforms");

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[uniforms.bind_group_entry(0)],
            label: Some("vger bind group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &Scene::bind_group_layout(&device),
                &uniform_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let blend_comp = wgpu::BlendComponent {
            operation: wgpu::BlendOperation::Add,
            src_factor: wgpu::BlendFactor::SrcAlpha,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState {
                        color: blend_comp,
                        alpha: blend_comp,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            scenes,
            cur_prim: [0, 0, 0, 0],
            cur_scene: 0,
            cur_layer: 0,
            tx_stack: vec![],
            device_px_ratio: 1.0,
            screen_size: ScreenSize::new(512.0, 512.0),
            paint_count: 0,
            pipeline,
            uniforms,
            uniform_bind_group,
            xform_count: 0,
        }
    }

    pub fn begin(&mut self, window_width: f32, window_height: f32, device_px_ratio: f32) {
        self.device_px_ratio = device_px_ratio;
        self.cur_prim = [0, 0, 0, 0];
        self.cur_layer = 0;
        self.screen_size = ScreenSize::new(window_width, window_height);
        self.uniforms[0] = Uniforms {
            size: [window_width, window_height],
        };
        self.cur_scene = (self.cur_scene + 1) % 3;
        self.tx_stack.clear();
        self.tx_stack.push(LocalToWorld::identity());
        self.paint_count = 0;
        self.xform_count = 0;
    }

    pub fn save(&mut self) {
        self.tx_stack.push(*self.tx_stack.last().unwrap())
    }

    pub fn restore(&mut self) {
        self.tx_stack.pop();
    }

    /// Encode all rendering to a command buffer.
    pub fn encode(
        &mut self,
        device: &wgpu::Device,
        render_pass: &wgpu::RenderPassDescriptor,
    ) -> wgpu::CommandBuffer {
        self.scenes[self.cur_scene].unmap();
        self.uniforms.unmap();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("vger encoder"),
        });

        {
            let mut rpass = encoder.begin_render_pass(render_pass);

            rpass.set_pipeline(&self.pipeline);

            rpass.set_bind_group(
                0,
                &self.scenes[self.cur_scene].bind_groups[self.cur_layer],
                &[], // dynamic offsets
            );

            rpass.set_bind_group(1, &self.uniform_bind_group, &[]);

            let n = self.cur_prim[self.cur_layer];
            println!("encoding {:?} prims", n);

            rpass.draw(/*vertices*/ 0..4, /*instances*/ 0..(n as u32))
        }
        encoder.finish()
    }

    fn render(&mut self, prim: Prim) {
        let prim_ix = self.cur_prim[self.cur_layer];
        if prim_ix < MAX_PRIMS {
            self.scenes[self.cur_scene].prims[self.cur_layer][prim_ix] = prim;
            self.cur_prim[self.cur_layer] += 1;
        }
    }

    pub fn fill_circle(&mut self, center: LocalPoint, radius: f32, paint_index: PaintIndex) {
        let mut prim = Prim::default();
        prim.prim_type = PrimType::Circle as u32;
        prim.cvs[0] = center.x;
        prim.cvs[1] = center.y;
        prim.radius = radius;
        prim.paint = paint_index.index as u32;
        prim.quad_bounds = [
            center.x - radius,
            center.y - radius,
            center.x + radius,
            center.y + radius,
        ];
        prim.tex_bounds = prim.quad_bounds;
        prim.xform = self.add_xform() as u32;

        self.render(prim);
    }

    pub fn stroke_arc(
        &mut self,
        center: LocalPoint,
        radius: f32,
        width: f32,
        rotation: f32,
        aperture: f32,
        paint_index: PaintIndex,
    ) {
        let mut prim = Prim::default();
        prim.prim_type = PrimType::Arc as u32;
        prim.radius = radius;
        prim.cvs = [
            center.x,
            center.y,
            rotation.sin(),
            rotation.cos(),
            aperture.sin(),
            aperture.cos(),
        ];
        prim.width = width;
        prim.paint = paint_index.index as u32;
        prim.xform = self.add_xform() as u32;

        self.render(prim);
    }

    pub fn fill_rect(
        &mut self,
        min: LocalPoint,
        max: LocalPoint,
        radius: f32,
        paint_index: PaintIndex,
    ) {
        let mut prim = Prim::default();
        prim.prim_type = PrimType::Rect as u32;
        prim.cvs[0] = min.x;
        prim.cvs[1] = min.y;
        prim.cvs[2] = max.x;
        prim.cvs[3] = max.y;
        prim.radius = radius;
        prim.paint = paint_index.index as u32;
        prim.quad_bounds = [min.x, min.y, max.x, max.y];
        prim.tex_bounds = prim.quad_bounds;
        prim.xform = self.add_xform() as u32;

        self.render(prim);
    }

    pub fn stroke_rect(
        &mut self,
        min: LocalPoint,
        max: LocalPoint,
        radius: f32,
        width: f32,
        paint_index: PaintIndex,
    ) {
        let mut prim = Prim::default();
        prim.prim_type = PrimType::RectStroke as u32;
        prim.cvs[0] = min.x;
        prim.cvs[1] = min.y;
        prim.cvs[2] = max.x;
        prim.cvs[3] = max.y;
        prim.radius = radius;
        prim.width = width;
        prim.paint = paint_index.index as u32;
        prim.quad_bounds = [min.x - width, min.y - width, max.x + width, max.y + width];
        prim.tex_bounds = prim.quad_bounds;
        prim.xform = self.add_xform() as u32;

        self.render(prim);
    }

    pub fn stroke_segment(
        &mut self,
        a: LocalPoint,
        b: LocalPoint,
        width: f32,
        paint_index: PaintIndex,
    ) {
        let mut prim = Prim::default();
        prim.prim_type = PrimType::Segment as u32;
        prim.cvs[0] = a.x;
        prim.cvs[1] = a.y;
        prim.cvs[2] = b.x;
        prim.cvs[3] = b.y;
        prim.width = width;
        prim.paint = paint_index.index as u32;
        prim.quad_bounds = [a.x.min(b.x), a.y.min(b.y), a.x.max(b.x), a.y.max(b.y)];
        prim.tex_bounds = prim.quad_bounds;
        prim.xform = self.add_xform() as u32;

        self.render(prim);
    }

    pub fn stroke_bezier(
        &mut self,
        a: LocalPoint,
        b: LocalPoint,
        c: LocalPoint,
        width: f32,
        paint_index: PaintIndex,
    ) {
        let mut prim = Prim::default();
        prim.prim_type = PrimType::Bezier as u32;
        prim.cvs[0] = a.x;
        prim.cvs[1] = a.y;
        prim.cvs[2] = b.x;
        prim.cvs[3] = b.y;
        prim.cvs[4] = c.x;
        prim.cvs[5] = c.y;
        prim.width = width;
        prim.paint = paint_index.index as u32;
        prim.quad_bounds = [a.x.min(b.x).min(c.x)-width, a.y.min(b.y).min(c.y)-width, a.x.max(b.x).max(c.x)+width, a.y.max(b.y).max(c.y)+width];
        prim.tex_bounds = prim.quad_bounds;
        prim.xform = self.add_xform() as u32;

        self.render(prim);
    }

    fn add_xform(&mut self) -> usize {
        if self.xform_count < MAX_PRIMS {
            self.scenes[self.cur_scene].xforms[self.xform_count] = *self.tx_stack.last().unwrap();
            let n = self.xform_count;
            self.xform_count += 1;
            return n;
        }
        0
    }

    pub fn translate(&mut self, offset: Vector2D<f32, LocalSpace>) {
        if let Some(m) = self.tx_stack.last_mut() {
            *m = (*m).pre_translate(offset);
        }
    }

    fn add_paint(&mut self, paint: Paint) -> PaintIndex {
        if self.paint_count < MAX_PRIMS {
            self.scenes[self.cur_scene].paints[self.paint_count] = paint;
            self.paint_count += 1;
            return PaintIndex {
                index: self.paint_count - 1,
            };
        }
        PaintIndex { index: 0 }
    }

    pub fn color_paint(&mut self, color: Color) -> PaintIndex {
        self.add_paint(Paint::solid_color(color))
    }

    pub fn linear_gradient(
        &mut self,
        start: LocalPoint,
        end: LocalPoint,
        inner_color: Color,
        outer_color: Color,
        glow: f32,
    ) -> PaintIndex {
        self.add_paint(Paint::linear_gradient(
            start,
            end,
            inner_color,
            outer_color,
            glow,
        ))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use futures::executor::block_on;
    use std::fs::File;
    use std::io::prelude::*;
    use std::mem::size_of;

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

    // See https://github.com/gfx-rs/wgpu/blob/master/wgpu/examples/capture/main.rs
    struct BufferDimensions {
        width: usize,
        height: usize,
        unpadded_bytes_per_row: usize,
        padded_bytes_per_row: usize,
    }

    impl BufferDimensions {
        fn new(width: usize, height: usize) -> Self {
            let bytes_per_pixel = size_of::<u32>();
            let unpadded_bytes_per_row = width * bytes_per_pixel;
            let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
            let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
            let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;
            Self {
                width,
                height,
                unpadded_bytes_per_row,
                padded_bytes_per_row,
            }
        }
    }

    async fn create_png(
        png_output_path: &str,
        device: &wgpu::Device,
        output_buffer: wgpu::Buffer,
        buffer_dimensions: &BufferDimensions,
    ) {
        // Note that we're not calling `.await` here.
        let buffer_slice = output_buffer.slice(..);
        let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);

        // Poll the device in a blocking manner so that our future resolves.
        // In an actual application, `device.poll(...)` should
        // be called in an event loop or on another thread.
        device.poll(wgpu::Maintain::Wait);
        // If a file system is available, write the buffer as a PNG
        let has_file_system_available = cfg!(not(target_arch = "wasm32"));
        if !has_file_system_available {
            return;
        }

        if let Ok(()) = buffer_future.await {
            let padded_buffer = buffer_slice.get_mapped_range();

            let mut png_encoder = png::Encoder::new(
                File::create(png_output_path).unwrap(),
                buffer_dimensions.width as u32,
                buffer_dimensions.height as u32,
            );
            png_encoder.set_depth(png::BitDepth::Eight);
            png_encoder.set_color(png::ColorType::RGBA);
            let mut png_writer = png_encoder
                .write_header()
                .unwrap()
                .into_stream_writer_with_size(buffer_dimensions.unpadded_bytes_per_row);

            // from the padded_buffer we write just the unpadded bytes into the image
            for chunk in padded_buffer.chunks(buffer_dimensions.padded_bytes_per_row) {
                png_writer
                    .write_all(&chunk[..buffer_dimensions.unpadded_bytes_per_row])
                    .unwrap();
            }
            png_writer.finish().unwrap();

            // With the current interface, we have to make sure all mapped views are
            // dropped before we unmap the buffer.
            drop(padded_buffer);

            output_buffer.unmap();
        }
    }

    fn render_test(vger: &mut VGER, device: &wgpu::Device, queue: &wgpu::Queue, name: &str) {
        let texture_size = wgpu::Extent3d {
            width: 512,
            height: 512,
            depth_or_array_layers: 1,
        };

        let render_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            label: Some("render_texture"),
        });

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 512 * 512 * 4,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let desc = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        };

        queue.submit(Some(vger.encode(device, &desc)));

        let buffer_dimensions = BufferDimensions::new(512, 512);

        let texture_extent = wgpu::Extent3d {
            width: buffer_dimensions.width as u32,
            height: buffer_dimensions.height as u32,
            depth_or_array_layers: 1,
        };

        let command_buffer = {
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            // Copy the data from the texture to the buffer
            encoder.copy_texture_to_buffer(
                render_texture.as_image_copy(),
                wgpu::ImageCopyBuffer {
                    buffer: &output_buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(
                            std::num::NonZeroU32::new(
                                buffer_dimensions.padded_bytes_per_row as u32,
                            )
                            .unwrap(),
                        ),
                        rows_per_image: None,
                    },
                },
                texture_extent,
            );

            encoder.finish()
        };

        queue.submit(Some(command_buffer));

        device.poll(wgpu::Maintain::Wait);

        device.stop_capture();

        block_on(create_png(name, device, output_buffer, &buffer_dimensions));
    }

    #[test]
    fn fill_circle() {
        let (device, queue) = block_on(setup());

        let mut vger = VGER::new(&device);

        vger.begin(512.0, 512.0, 1.0);
        let cyan = vger.color_paint(Color::CYAN);
        vger.fill_circle([100.0, 100.0].into(), 20.0, cyan);

        render_test(&mut vger, &device, &queue, "circle.png");
    }

    #[test]
    fn fill_rect() {
        let (device, queue) = block_on(setup());

        let mut vger = VGER::new(&device);

        vger.begin(512.0, 512.0, 1.0);
        let cyan = vger.color_paint(Color::CYAN);
        vger.fill_rect([100.0, 100.0].into(), [200.0, 200.0].into(), 10.0, cyan);

        render_test(&mut vger, &device, &queue, "rect.png");
    }

    #[test]
    fn fill_rect_gradient() {
        let (device, queue) = block_on(setup());

        let mut vger = VGER::new(&device);

        vger.begin(512.0, 512.0, 1.0);

        // vgerLinearGradient(vger, float2{50,450}, float2{100,450}, cyan, magenta, 0)
        let paint = vger.linear_gradient(
            [100.0, 100.0].into(),
            [200.0, 200.0].into(),
            Color::CYAN,
            Color::MAGENTA,
            0.0,
        );

        vger.fill_rect([100.0, 100.0].into(), [200.0, 200.0].into(), 10.0, paint);

        render_test(&mut vger, &device, &queue, "rect_gradient.png");
    }

    #[test]
    fn stroke_rect_gradient() {
        let (device, queue) = block_on(setup());

        let mut vger = VGER::new(&device);

        vger.begin(512.0, 512.0, 1.0);

        let paint = vger.linear_gradient(
            [100.0, 100.0].into(),
            [200.0, 200.0].into(),
            Color::CYAN,
            Color::MAGENTA,
            0.0,
        );

        vger.stroke_rect(
            [100.0, 100.0].into(),
            [200.0, 200.0].into(),
            10.0,
            4.0,
            paint,
        );

        render_test(&mut vger, &device, &queue, "rect_stroke_gradient.png");
    }

    #[test]
    fn stroke_arc_gradient() {
        let (device, queue) = block_on(setup());

        let mut vger = VGER::new(&device);

        vger.begin(512.0, 512.0, 1.0);

        let paint = vger.linear_gradient(
            [100.0, 100.0].into(),
            [300.0, 300.0].into(),
            Color::CYAN,
            Color::MAGENTA,
            0.0,
        );

        vger.stroke_arc([200.0,200.0].into(), 100.0, 4.0, 0.0, std::f32::consts::PI / 4.0, paint);

        render_test(&mut vger, &device, &queue, "arc_stroke_gradient.png");
    }

    #[test]
    fn segment_stroke_gradient() {
        let (device, queue) = block_on(setup());

        let mut vger = VGER::new(&device);

        vger.begin(512.0, 512.0, 1.0);

        let paint = vger.linear_gradient(
            [100.0, 100.0].into(),
            [200.0, 200.0].into(),
            Color::CYAN,
            Color::MAGENTA,
            0.0,
        );

        vger.stroke_segment(
            [100.0, 100.0].into(),
            [200.0, 200.0].into(),
            4.0,
            paint,
        );

        render_test(&mut vger, &device, &queue, "segment_stroke_gradient.png");
    }

    #[test]
    fn bezier_stroke_gradient() {
        let (device, queue) = block_on(setup());

        let mut vger = VGER::new(&device);

        vger.begin(512.0, 512.0, 1.0);

        let paint = vger.linear_gradient(
            [100.0, 100.0].into(),
            [200.0, 200.0].into(),
            Color::CYAN,
            Color::MAGENTA,
            0.0,
        );

        vger.stroke_bezier(
            [100.0, 100.0].into(),
            [150.0, 200.0].into(),
            [200.0, 200.0].into(),
            4.0,
            paint,
        );

        render_test(&mut vger, &device, &queue, "bezier_stroke_gradient.png");
    }
}
