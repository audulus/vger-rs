use cosmic_text::{SubpixelBin, SwashImage};
use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use std::sync::Arc;

mod path;
use path::*;

mod scene;
use scene::*;

mod prim;
use prim::*;

pub mod defs;
use defs::*;

mod paint;
use paint::*;

mod gpu_vec;
use gpu_vec::*;

pub mod color;
pub use color::Color;

pub mod atlas;

mod glyphs;
use glyphs::GlyphCache;

use wgpu::util::DeviceExt;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
struct Uniforms {
    size: [f32; 2],
}

#[derive(Copy, Clone, Debug)]
pub struct PaintIndex {
    index: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct ImageIndex {
    index: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct LineMetrics {
    pub glyph_start: usize,
    pub glyph_end: usize,
    pub bounds: LocalRect,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub(crate) struct Scissor {
    pub xform: WorldToLocal,
    pub origin: [f32; 2],
    pub size: [f32; 2],
}

impl Scissor {
    fn new() -> Self {
        Self {
            xform: WorldToLocal::identity(),
            origin: [-10000.0, -10000.0],
            size: [20000.0, 20000.0],
        }
    }
}

pub struct Vger {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    scenes: [Scene; 3],
    cur_scene: usize,
    cur_layer: usize,
    tx_stack: Vec<LocalToWorld>,
    scissor_stack: Vec<Scissor>,
    device_px_ratio: f32,
    screen_size: ScreenSize,
    paint_count: usize,
    pipeline: wgpu::RenderPipeline,
    uniform_bind_group: wgpu::BindGroup,
    uniforms: GPUVec<Uniforms>,
    xform_count: usize,
    scissor_count: usize,
    path_scanner: PathScanner,
    pen: LocalPoint,
    pub glyph_cache: GlyphCache,
    layout: Layout,
    images: Vec<Option<wgpu::Texture>>,
    image_bind_groups: Vec<Option<wgpu::BindGroup>>,
    image_bind_group_layout: wgpu::BindGroupLayout,
    default_image_bind_group: wgpu::BindGroup,
}

impl Vger {
    /// Create a new renderer given a device and output pixel format.
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
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
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("uniform_bind_group_layout"),
            });

        let image_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
                label: Some("image_bind_group_layout"),
            });

        let glyph_cache = GlyphCache::new(&device);

        let mask_texture_view = glyph_cache.mask_atlas.create_view();
        let color_texture_view = glyph_cache.color_atlas.create_view();

        let uniforms = GPUVec::new_uniforms(&device, "uniforms");

        let glyph_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("glyph"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                uniforms.bind_group_entry(0),
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&mask_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&color_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&glyph_sampler),
                },
            ],
            label: Some("vger bind group"),
        });

        let default_image_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &image_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&mask_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&color_texture_view),
                },
            ],
            label: Some("vger default image bind group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &Scene::bind_group_layout(&device),
                &uniform_bind_group_layout,
                &image_bind_group_layout,
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
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState {
                        color: blend_comp,
                        alpha: blend_comp,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
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

        let layout = Layout::new(CoordinateSystem::PositiveYUp);

        Self {
            device,
            queue,
            scenes,
            cur_scene: 0,
            cur_layer: 0,
            tx_stack: vec![],
            scissor_stack: vec![],
            device_px_ratio: 1.0,
            screen_size: ScreenSize::new(512.0, 512.0),
            paint_count: 0,
            pipeline,
            uniforms,
            uniform_bind_group,
            xform_count: 0,
            scissor_count: 0,
            path_scanner: PathScanner::new(),
            pen: LocalPoint::zero(),
            glyph_cache,
            layout,
            images: vec![],
            image_bind_groups: vec![],
            image_bind_group_layout,
            default_image_bind_group,
        }
    }

    /// Begin rendering.
    pub fn begin(&mut self, window_width: f32, window_height: f32, device_px_ratio: f32) {
        self.device_px_ratio = device_px_ratio;
        self.cur_layer = 0;
        self.screen_size = ScreenSize::new(window_width, window_height);
        self.uniforms.clear();
        self.uniforms.push(Uniforms {
            size: [window_width, window_height],
        });
        self.cur_scene = (self.cur_scene + 1) % 3;
        self.scenes[self.cur_scene].clear();
        self.tx_stack.clear();
        self.tx_stack.push(LocalToWorld::identity());
        self.scissor_stack.clear();
        self.scissor_stack.push(Scissor::new());
        self.paint_count = 0;
        self.xform_count = 0;
        self.scissor_count = 0;
        self.pen = LocalPoint::zero();
    }

    /// Saves rendering state (transform and scissor rect).
    pub fn save(&mut self) {
        self.tx_stack.push(*self.tx_stack.last().unwrap());
        self.scissor_stack.push(*self.scissor_stack.last().unwrap());
    }

    /// Restores rendering state (transform and scissor rect).
    pub fn restore(&mut self) {
        self.tx_stack.pop();
        self.scissor_stack.pop();
    }

    /// Encode all rendering to a command buffer.
    pub fn encode(&mut self, render_pass: &wgpu::RenderPassDescriptor) {
        let device = &self.device;
        let queue = &self.queue;
        self.scenes[self.cur_scene].update(device, queue);
        self.uniforms.update(device, queue);
        let mut current_texture = -1;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("vger encoder"),
        });

        self.glyph_cache.update(device, &mut encoder);

        {
            let mut rpass = encoder.begin_render_pass(render_pass);

            rpass.set_pipeline(&self.pipeline);

            rpass.set_bind_group(
                0,
                &self.scenes[self.cur_scene].bind_groups[self.cur_layer],
                &[], // dynamic offsets
            );

            rpass.set_bind_group(1, &self.uniform_bind_group, &[]);
            rpass.set_bind_group(2, &self.default_image_bind_group, &[]);

            let scene = &self.scenes[self.cur_scene];
            let n = scene.prims[self.cur_layer].len();
            let mut m: u32 = 0;
            let mut start: u32 = 0;

            for i in 0..n {
                let prim = &scene.prims[self.cur_layer][i];
                let image_id = scene.paints[prim.paint as usize].image;

                // Image changed, render.
                if image_id >= 0 && image_id != current_texture {
                    // println!("image changed: encoding {:?} prims", m);
                    if m > 0 {
                        rpass.draw(
                            /*vertices*/ 0..4,
                            /*instances*/ start..(start + m),
                        );
                    }

                    current_texture = image_id;
                    rpass.set_bind_group(
                        2,
                        self.image_bind_groups[image_id as usize].as_ref().unwrap(),
                        &[],
                    );

                    start += m;
                    m = 0;
                }

                m += 1;
            }

            // println!("encoding {:?} prims", m);

            if m > 0 {
                rpass.draw(
                    /*vertices*/ 0..4,
                    /*instances*/ start..(start + m),
                )
            }
        }
        queue.submit(Some(encoder.finish()));

        // If we're getting close to full, reset the glyph cache.
        self.glyph_cache.check_usage();
    }

    fn render(&mut self, prim: Prim) {
        self.scenes[self.cur_scene].prims[self.cur_layer].push(prim);
    }

    /// Fills a circle.
    pub fn fill_circle<Pt: Into<LocalPoint>>(
        &mut self,
        center: Pt,
        radius: f32,
        paint_index: PaintIndex,
    ) {
        let mut prim = Prim::default();
        prim.prim_type = PrimType::Circle as u32;
        let c: LocalPoint = center.into();
        prim.cvs[0] = c.x;
        prim.cvs[1] = c.y;
        prim.radius = radius;
        prim.paint = paint_index.index as u32;
        prim.quad_bounds = [c.x - radius, c.y - radius, c.x + radius, c.y + radius];
        prim.tex_bounds = prim.quad_bounds;
        prim.xform = self.add_xform() as u32;
        prim.scissor = self.add_scissor() as u32;

        self.render(prim);
    }

    /// Strokes an arc.
    pub fn stroke_arc<Pt: Into<LocalPoint>>(
        &mut self,
        center: Pt,
        radius: f32,
        width: f32,
        rotation: f32,
        aperture: f32,
        paint_index: PaintIndex,
    ) {
        let mut prim = Prim::default();
        prim.prim_type = PrimType::Arc as u32;
        prim.radius = radius;
        let c: LocalPoint = center.into();
        prim.cvs = [
            c.x,
            c.y,
            rotation.sin(),
            rotation.cos(),
            aperture.sin(),
            aperture.cos(),
        ];
        prim.width = width;
        prim.paint = paint_index.index as u32;
        prim.quad_bounds = [
            c.x - radius - width,
            c.y - radius - width,
            c.x + radius + width,
            c.y + radius + width,
        ];
        prim.tex_bounds = prim.quad_bounds;
        prim.xform = self.add_xform() as u32;
        prim.scissor = self.add_scissor() as u32;

        self.render(prim);
    }

    /// Fills a rectangle.
    pub fn fill_rect<Rect: Into<LocalRect>>(
        &mut self,
        rect: Rect,
        radius: f32,
        paint_index: PaintIndex,
    ) {
        let mut prim = Prim::default();
        prim.prim_type = PrimType::Rect as u32;
        let r: LocalRect = rect.into();
        let min = r.min();
        let max = r.max();
        prim.cvs[0] = min.x;
        prim.cvs[1] = min.y;
        prim.cvs[2] = max.x;
        prim.cvs[3] = max.y;
        prim.radius = radius;
        prim.paint = paint_index.index as u32;
        prim.quad_bounds = [min.x, min.y, max.x, max.y];
        prim.tex_bounds = prim.quad_bounds;
        prim.xform = self.add_xform() as u32;
        prim.scissor = self.add_scissor() as u32;

        self.render(prim);
    }

    /// Strokes a rectangle.
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
        prim.scissor = self.add_scissor() as u32;

        self.render(prim);
    }

    /// Strokes a line segment.
    pub fn stroke_segment<Pt: Into<LocalPoint>>(
        &mut self,
        a: Pt,
        b: Pt,
        width: f32,
        paint_index: PaintIndex,
    ) {
        let mut prim = Prim::default();
        prim.prim_type = PrimType::Segment as u32;
        let ap: LocalPoint = a.into();
        let bp: LocalPoint = b.into();
        prim.cvs[0] = ap.x;
        prim.cvs[1] = ap.y;
        prim.cvs[2] = bp.x;
        prim.cvs[3] = bp.y;
        prim.width = width;
        prim.paint = paint_index.index as u32;
        prim.quad_bounds = [
            ap.x.min(bp.x) - width / 2.0,
            ap.y.min(bp.y) - width / 2.0,
            ap.x.max(bp.x) + width / 2.0,
            ap.y.max(bp.y) + width / 2.0,
        ];
        prim.tex_bounds = prim.quad_bounds;
        prim.xform = self.add_xform() as u32;
        prim.scissor = self.add_scissor() as u32;

        self.render(prim);
    }

    /// Strokes a quadratic bezier segment.
    pub fn stroke_bezier<Pt: Into<LocalPoint>>(
        &mut self,
        a: Pt,
        b: Pt,
        c: Pt,
        width: f32,
        paint_index: PaintIndex,
    ) {
        let mut prim = Prim::default();
        prim.prim_type = PrimType::Bezier as u32;
        let ap: LocalPoint = a.into();
        let bp: LocalPoint = b.into();
        let cp: LocalPoint = c.into();
        prim.cvs[0] = ap.x;
        prim.cvs[1] = ap.y;
        prim.cvs[2] = bp.x;
        prim.cvs[3] = bp.y;
        prim.cvs[4] = cp.x;
        prim.cvs[5] = cp.y;
        prim.width = width;
        prim.paint = paint_index.index as u32;
        prim.quad_bounds = [
            ap.x.min(bp.x).min(cp.x) - width,
            ap.y.min(bp.y).min(cp.y) - width,
            ap.x.max(bp.x).max(cp.x) + width,
            ap.y.max(bp.y).max(cp.y) + width,
        ];
        prim.tex_bounds = prim.quad_bounds;
        prim.xform = self.add_xform() as u32;
        prim.scissor = self.add_scissor() as u32;

        self.render(prim);
    }

    /// Move the pen to a point (path fills only)
    pub fn move_to<Pt: Into<LocalPoint>>(&mut self, p: Pt) {
        self.pen = p.into();
    }

    /// Makes a quadratic curve to a point (path fills only)
    pub fn quad_to<Pt: Into<LocalPoint>>(&mut self, b: Pt, c: Pt) {
        let cp: LocalPoint = c.into();
        self.path_scanner
            .segments
            .push(PathSegment::new(self.pen, b.into(), cp));
        self.pen = cp;
    }

    fn add_cv<Pt: Into<LocalPoint>>(&mut self, p: Pt) {
        self.scenes[self.cur_scene].cvs.push(p.into())
    }

    /// Fills a path.
    pub fn fill(&mut self, paint_index: PaintIndex) {
        let xform = self.add_xform();
        let scissor = self.add_scissor();

        self.path_scanner.init();

        while self.path_scanner.next() {
            let mut prim = Prim::default();
            prim.prim_type = PrimType::PathFill as u32;
            prim.paint = paint_index.index as u32;
            prim.xform = xform as u32;
            prim.scissor = scissor as u32;
            prim.start = self.scenes[self.cur_scene].cvs.len() as u32;

            let mut x_interval = Interval {
                a: std::f32::MAX,
                b: std::f32::MIN,
            };

            let mut index = self.path_scanner.first;
            while let Some(a) = index {
                for i in 0..3 {
                    let p = self.path_scanner.segments[a].cvs[i];
                    self.add_cv(p);
                    x_interval.a = x_interval.a.min(p.x);
                    x_interval.b = x_interval.b.max(p.x);
                }
                prim.count += 1;

                index = self.path_scanner.segments[a].next;
            }

            prim.quad_bounds[0] = x_interval.a;
            prim.quad_bounds[1] = self.path_scanner.interval.a;
            prim.quad_bounds[2] = x_interval.b;
            prim.quad_bounds[3] = self.path_scanner.interval.b;
            prim.tex_bounds = prim.quad_bounds;

            self.render(prim);
        }
    }

    fn setup_layout(&mut self, text: &str, size: u32, max_width: Option<f32>) {
        let scale = self.device_px_ratio;

        self.layout.reset(&LayoutSettings {
            max_width: max_width.map(|w| w * scale),
            ..LayoutSettings::default()
        });

        let scaled_size = size as f32 * scale;

        self.layout.append(
            &[&self.glyph_cache.font],
            &TextStyle::new(text, scaled_size, 0),
        );
    }

    pub fn render_glyph<'a>(
        &mut self,
        x: f32,
        y: f32,
        font_id: cosmic_text::fontdb::ID,
        glyph_id: u16,
        size: u32,
        subpx: SubpixelBin,
        image: impl FnOnce() -> SwashImage,
        paint_index: PaintIndex,
    ) {
        let info = self
            .glyph_cache
            .get_glyph_mask(font_id, glyph_id, size, subpx, image);
        if let Some(rect) = info.rect {
            let mut prim = Prim::default();
            prim.prim_type = if info.colored {
                PrimType::ColorGlyph
            } else {
                PrimType::Glyph
            } as u32;

            let x = x + info.left as f32;
            let y = y - info.top as f32;
            prim.quad_bounds = [x, y, x + rect.width as f32, y + rect.height as f32];

            prim.tex_bounds = [
                rect.x as f32,
                rect.y as f32,
                (rect.x + rect.width) as f32,
                (rect.y + rect.height) as f32,
            ];
            prim.paint = paint_index.index as u32;

            self.render(prim);
        }
    }

    pub fn render_svg(
        &mut self,
        x: f32,
        y: f32,
        hash: &[u8],
        width: u32,
        height: u32,
        image: impl FnOnce() -> Vec<u8>,
        paint_index: Option<PaintIndex>,
    ) {
        let info = self.glyph_cache.get_svg_mask(hash, width, height, image);
        if let Some(rect) = info.rect {
            let mut prim = Prim::default();
            prim.prim_type = if info.colored {
                PrimType::ColorGlyph
            } else {
                PrimType::Glyph
            } as u32;

            let x = x + info.left as f32;
            let y = y - info.top as f32;
            prim.quad_bounds = [x, y, x + rect.width as f32, y + rect.height as f32];

            prim.tex_bounds = [
                rect.x as f32,
                rect.y as f32,
                (rect.x + rect.width) as f32,
                (rect.y + rect.height) as f32,
            ];
            if let Some(paint_index) = paint_index {
                prim.paint = paint_index.index as u32;
            }

            self.render(prim);
        }
    }

    /// Renders text.
    pub fn text(&mut self, text: &str, size: u32, color: Color, max_width: Option<f32>) {
        self.setup_layout(text, size, max_width);

        let scale = self.device_px_ratio;
        let scaled_size = size as f32 * scale;

        let paint = self.color_paint(color);
        let xform = self.add_xform() as u32;
        let scissor = self.add_scissor() as u32;

        let mut prims = vec![];
        for (i, glyph) in self.layout.glyphs().iter().enumerate() {
            let c = text.chars().nth(i).unwrap();
            // println!("glyph {:?}", c);
            let info = self.glyph_cache.get_glyph(c, scaled_size);

            if let Some(rect) = info.rect {
                let mut prim = Prim::default();
                prim.prim_type = PrimType::Glyph as u32;
                prim.xform = xform;
                prim.scissor = scissor;
                assert!(glyph.width == rect.width as usize);
                assert!(glyph.height == rect.height as usize);

                prim.quad_bounds = [
                    glyph.x / scale,
                    glyph.y / scale,
                    (glyph.x + glyph.width as f32) / scale,
                    (glyph.y + glyph.height as f32) / scale,
                ];
                // println!("quad_bounds: {:?}", prim.quad_bounds);

                prim.tex_bounds = [
                    rect.x as f32,
                    (rect.y + rect.height) as f32,
                    (rect.x + rect.width) as f32,
                    rect.y as f32,
                ];
                prim.paint = paint.index as u32;
                // println!("tex_bounds: {:?}", prim.tex_bounds);

                prims.push(prim);
            }
        }

        for prim in prims {
            self.render(prim);
        }
    }

    /// Calculates the bounds for text.
    pub fn text_bounds(&mut self, text: &str, size: u32, max_width: Option<f32>) -> LocalRect {
        self.setup_layout(text, size, max_width);

        let mut min = LocalPoint::new(f32::MAX, f32::MAX);
        let mut max = LocalPoint::new(f32::MIN, f32::MIN);

        let scale = self.device_px_ratio;

        for glyph in self.layout.glyphs() {
            min = min.min([glyph.x / scale, glyph.y / scale].into());
            max = max.max(
                [
                    (glyph.x + glyph.width as f32) / scale,
                    (glyph.y + glyph.height as f32) / scale,
                ]
                .into(),
            );
        }

        LocalRect::new(min, (max - min).into())
    }

    /// Returns local coordinates of glyphs.
    pub fn glyph_positions(
        &mut self,
        text: &str,
        size: u32,
        max_width: Option<f32>,
    ) -> Vec<LocalRect> {
        let mut rects = vec![];
        rects.reserve(text.len());

        self.setup_layout(text, size, max_width);

        let s = 1.0 / self.device_px_ratio;

        for glyph in self.layout.glyphs() {
            rects.push(
                LocalRect::new(
                    [glyph.x, glyph.y].into(),
                    [glyph.width as f32, glyph.height as f32].into(),
                )
                .scale(s, s),
            )
        }

        rects
    }

    pub fn line_metrics(
        &mut self,
        text: &str,
        size: u32,
        max_width: Option<f32>,
    ) -> Vec<LineMetrics> {
        self.setup_layout(text, size, max_width);
        let s = 1.0 / self.device_px_ratio;

        let mut rects = vec![];
        rects.reserve(text.len());

        let glyphs = self.layout.glyphs();

        if let Some(lines) = self.layout.lines() {
            for line in lines {
                let mut rect = LocalRect::zero();

                for glyph in &glyphs[line.glyph_start..line.glyph_end] {
                    rect = rect.union(&LocalRect::new(
                        [glyph.x, glyph.y].into(),
                        [glyph.width as f32, glyph.height as f32].into(),
                    ));
                }
                rects.push(LineMetrics {
                    glyph_start: line.glyph_start,
                    glyph_end: line.glyph_end,
                    bounds: rect.scale(s, s),
                });
            }
        }

        rects
    }

    fn add_xform(&mut self) -> usize {
        if self.xform_count < MAX_PRIMS {
            let m = *self.tx_stack.last().unwrap();
            self.scenes[self.cur_scene]
                .xforms
                .push(m.to_3d().to_array());
            let n = self.xform_count;
            self.xform_count += 1;
            return n;
        }
        0
    }

    fn add_scissor(&mut self) -> usize {
        if self.scissor_count < MAX_PRIMS {
            let scissor = *self.scissor_stack.last().unwrap();
            self.scenes[self.cur_scene].scissors.push(scissor);
            let n = self.scissor_count;
            self.scissor_count += 1;
            return n;
        }
        0
    }

    /// Translates the coordinate system.
    pub fn translate<Vec: Into<LocalVector>>(&mut self, offset: Vec) {
        if let Some(m) = self.tx_stack.last_mut() {
            *m = (*m).pre_translate(offset.into());
        }
    }

    /// Scales the coordinate system.
    pub fn scale<Vec: Into<LocalVector>>(&mut self, scale: Vec) {
        if let Some(m) = self.tx_stack.last_mut() {
            let s: LocalVector = scale.into();
            *m = (*m).pre_scale(s.x, s.y);
        }
    }

    /// Rotates the coordinate system.
    pub fn rotate(&mut self, theta: f32) {
        if let Some(m) = self.tx_stack.last_mut() {
            *m = m.pre_rotate(euclid::Angle::<f32>::radians(theta));
        }
    }

    /// Gets the current transform.
    pub fn current_transform(&self) -> LocalToWorld {
        *self.tx_stack.last().unwrap()
    }

    /// Sets the current scissor rect.
    pub fn scissor(&mut self, rect: LocalRect) {
        if let Some(m) = self.scissor_stack.last_mut() {
            *m = Scissor::new();
            if let Some(xform) = self.tx_stack.last().unwrap().inverse() {
                m.xform = xform;
                m.origin = rect.origin.to_array();
                m.size = rect.size.to_array();
            }
        }
    }

    /// Resets the current scissor rect.
    pub fn reset_scissor(&mut self) {
        if let Some(m) = self.scissor_stack.last_mut() {
            *m = Scissor::new();
        }
    }

    fn add_paint(&mut self, paint: Paint) -> PaintIndex {
        if self.paint_count < MAX_PRIMS {
            self.scenes[self.cur_scene].paints.push(paint);
            self.paint_count += 1;
            return PaintIndex {
                index: self.paint_count - 1,
            };
        }
        PaintIndex { index: 0 }
    }

    /// Solid color paint.
    pub fn color_paint(&mut self, color: Color) -> PaintIndex {
        self.add_paint(Paint::solid_color(color))
    }

    /// Linear gradient paint.
    pub fn linear_gradient<Pt: Into<LocalPoint>>(
        &mut self,
        start: Pt,
        end: Pt,
        inner_color: Color,
        outer_color: Color,
        glow: f32,
    ) -> PaintIndex {
        self.add_paint(Paint::linear_gradient(
            start.into(),
            end.into(),
            inner_color,
            outer_color,
            glow,
        ))
    }

    /// Create an image from pixel data in memory.
    /// Must be RGBA8.
    pub fn create_image_pixels(&mut self, data: &[u8], width: u32, height: u32) -> ImageIndex {
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture_desc = wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            label: Some("lyte image"),
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        };

        let texture = self.device.create_texture(&texture_desc);

        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Temp Buffer"),
                contents: data,
                usage: wgpu::BufferUsages::COPY_SRC,
            });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("texture_buffer_copy_encoder"),
            });

        let image_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        encoder.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer: &buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 4),
                    rows_per_image: Some(height),
                },
            },
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                aspect: wgpu::TextureAspect::All,
                origin: wgpu::Origin3d::ZERO,
            },
            image_size,
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        let index = ImageIndex {
            index: self.images.len(),
        };

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.images.push(Some(texture));

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.image_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            }],
            label: Some("vger bind group"),
        });

        self.image_bind_groups.push(Some(bind_group));

        index
    }

    pub fn delete_image(&mut self, image: ImageIndex) {
        self.images[image.index] = None;
        self.image_bind_groups[image.index] = None;
    }
}

#[derive(Hash, Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum SubpixelOffset {
    Zero = 0,
    Quarter = 1,
    Half = 2,
    ThreeQuarters = 3,
}

impl Default for SubpixelOffset {
    fn default() -> Self {
        SubpixelOffset::Zero
    }
}

impl SubpixelOffset {
    // Skia quantizes subpixel offsets into 1/4 increments.
    // Given the absolute position, return the quantized increment
    pub fn quantize(pos: f32) -> Self {
        // Following the conventions of Gecko and Skia, we want
        // to quantize the subpixel position, such that abs(pos) gives:
        // [0.0, 0.125) -> Zero
        // [0.125, 0.375) -> Quarter
        // [0.375, 0.625) -> Half
        // [0.625, 0.875) -> ThreeQuarters,
        // [0.875, 1.0) -> Zero
        // The unit tests below check for this.
        let apos = ((pos - pos.floor()) * 8.0) as i32;
        match apos {
            1..=2 => SubpixelOffset::Quarter,
            3..=4 => SubpixelOffset::Half,
            5..=6 => SubpixelOffset::ThreeQuarters,
            _ => SubpixelOffset::Zero,
        }
    }

    pub fn to_f32(self) -> f32 {
        match self {
            SubpixelOffset::Zero => 0.0,
            SubpixelOffset::Quarter => 0.25,
            SubpixelOffset::Half => 0.5,
            SubpixelOffset::ThreeQuarters => 0.75,
        }
    }
}
