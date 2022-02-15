use euclid::*;
use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};

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
use color::Color;

mod atlas;

mod glyphs;
use glyphs::GlyphCache;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
struct Uniforms {
    size: [f32; 2],
}

#[derive(Copy, Clone, Debug)]
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
    path_scanner: PathScanner,
    pen: LocalPoint,
    cv_count: usize,
    glyph_cache: GlyphCache,
    layout: Layout,
}

impl VGER {
    pub fn new(device: &wgpu::Device, texture_format: wgpu::TextureFormat) -> Self {
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
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("uniform_bind_group_layout"),
            });

        let glyph_cache = GlyphCache::new(device);

        let texture_view = glyph_cache.create_view();

        let uniforms = GPUVec::new_uniforms(device, "uniforms");

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
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&glyph_sampler),
                },
            ],
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
                    format: texture_format,
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

        let layout = Layout::new(CoordinateSystem::PositiveYUp);

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
            path_scanner: PathScanner::new(),
            pen: LocalPoint::zero(),
            cv_count: 0,
            glyph_cache,
            layout,
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
        self.pen = LocalPoint::zero();
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
        prim.quad_bounds = [
            a.x.min(b.x).min(c.x) - width,
            a.y.min(b.y).min(c.y) - width,
            a.x.max(b.x).max(c.x) + width,
            a.y.max(b.y).max(c.y) + width,
        ];
        prim.tex_bounds = prim.quad_bounds;
        prim.xform = self.add_xform() as u32;

        self.render(prim);
    }

    pub fn move_to(&mut self, p: LocalPoint) {
        self.pen = p;
    }

    pub fn quad_to(&mut self, b: LocalPoint, c: LocalPoint) {
        self.path_scanner
            .segments
            .push(PathSegment::new(self.pen, b, c));
        self.pen = c;
    }

    fn add_cv(&mut self, p: LocalPoint) {
        if self.cv_count < MAX_PRIMS {
            self.scenes[self.cur_scene].cvs[self.cv_count] = p;
            self.cv_count += 1;
        }
    }

    pub fn fill(&mut self, paint_index: PaintIndex) {
        let xform = self.add_xform();

        self.path_scanner.init();

        while self.path_scanner.next() {
            let mut prim = Prim::default();
            prim.prim_type = PrimType::PathFill as u32;
            prim.paint = paint_index.index as u32;
            prim.xform = xform as u32;
            prim.start = self.cv_count as u32;

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

    pub fn text(&mut self, text: &str, size: u32, max_width: Option<f32>) {
        self.layout.reset(&LayoutSettings {
            max_width,
            ..LayoutSettings::default()
        });

        self.layout.append(
            &[&self.glyph_cache.font],
            &TextStyle::new(text, size as f32, 0),
        );

        let xform = self.add_xform() as u32;

        let mut i = 0;
        let mut prims = vec![];
        for glyph in self.layout.glyphs() {
            let c = text.chars().nth(i).unwrap();
            println!("glyph {:?}", c);
            let info = self.glyph_cache.get_glyph(c, size);

            if let Some(rect) = info.rect {
                let mut prim = Prim::default();
                prim.prim_type = PrimType::Glyph as u32;
                prim.xform = xform;
                prim.quad_bounds = [
                    glyph.x,
                    glyph.y,
                    glyph.x + glyph.width as f32,
                    glyph.y + glyph.height as f32,
                ];
                println!("quad_bounds: {:?}", prim.quad_bounds);
                prim.tex_bounds = [
                    rect.x as f32,
                    (rect.y + rect.height) as f32,
                    (rect.x + rect.width) as f32,
                    rect.y as f32,
                ];
                println!("tex_bounds: {:?}", prim.tex_bounds);

                prims.push(prim);
            }

            i += 1;
        }

        for prim in prims {
            self.render(prim);
        }
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
