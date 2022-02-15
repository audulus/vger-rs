
use vger::*;
use vger::color::Color;
use vger::defs::*;
use futures::executor::block_on;
use std::fs::File;
use std::io::prelude::*;
use std::mem::size_of;
extern crate rand;

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

fn render_test(
    vger: &mut VGER,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    name: &str,
    capture: bool,
) {
    if capture {
        device.start_capture();
    }

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

    if capture {
        device.stop_capture();
    }

    block_on(create_png(name, device, output_buffer, &buffer_dimensions));
}

#[test]
fn fill_circle() {
    let (device, queue) = block_on(setup());

    let mut vger = VGER::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb);

    vger.begin(512.0, 512.0, 1.0, &device);
    let cyan = vger.color_paint(Color::CYAN);
    vger.fill_circle([100.0, 100.0].into(), 20.0, cyan);

    render_test(&mut vger, &device, &queue, "circle.png", false);
}

#[test]
fn fill_circle_array() {
    let (device, queue) = block_on(setup());

    let mut vger = VGER::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb);

    vger.begin(512.0, 512.0, 1.0, &device);
    let cyan = vger.color_paint(Color::CYAN);

    for i in 0..5 {
        vger.fill_circle([100.0 * (i as f32), 100.0].into(), 20.0, cyan);
    }

    render_test(&mut vger, &device, &queue, "circle_array.png", false);
}

#[test]
fn fill_circle_translate() {
    let (device, queue) = block_on(setup());

    let mut vger = VGER::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb);

    vger.begin(512.0, 512.0, 1.0, &device);
    let cyan = vger.color_paint(Color::CYAN);
    vger.translate([256.0, 256.0].into());
    vger.fill_circle([0.0, 0.0].into(), 20.0, cyan);

    render_test(&mut vger, &device, &queue, "circle_translate.png", false);
}

#[test]
fn fill_rect() {
    let (device, queue) = block_on(setup());

    let mut vger = VGER::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb);

    vger.begin(512.0, 512.0, 1.0, &device);
    let cyan = vger.color_paint(Color::CYAN);
    vger.fill_rect([100.0, 100.0].into(), [200.0, 200.0].into(), 10.0, cyan);

    render_test(&mut vger, &device, &queue, "rect.png", false);
}

#[test]
fn fill_rect_gradient() {
    let (device, queue) = block_on(setup());

    let mut vger = VGER::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb);

    vger.begin(512.0, 512.0, 1.0, &device);

    let paint = vger.linear_gradient(
        [100.0, 100.0].into(),
        [200.0, 200.0].into(),
        Color::CYAN,
        Color::MAGENTA,
        0.0,
    );

    vger.fill_rect([100.0, 100.0].into(), [200.0, 200.0].into(), 10.0, paint);

    render_test(&mut vger, &device, &queue, "rect_gradient.png", false);
}

#[test]
fn stroke_rect_gradient() {
    let (device, queue) = block_on(setup());

    let mut vger = VGER::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb);

    vger.begin(512.0, 512.0, 1.0, &device);

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

    render_test(
        &mut vger,
        &device,
        &queue,
        "rect_stroke_gradient.png",
        false,
    );
}

#[test]
fn stroke_arc_gradient() {
    let (device, queue) = block_on(setup());

    let mut vger = VGER::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb);

    vger.begin(512.0, 512.0, 1.0, &device);

    let paint = vger.linear_gradient(
        [100.0, 100.0].into(),
        [300.0, 300.0].into(),
        Color::CYAN,
        Color::MAGENTA,
        0.0,
    );

    vger.stroke_arc(
        [200.0, 200.0].into(),
        100.0,
        4.0,
        0.0,
        std::f32::consts::PI / 4.0,
        paint,
    );

    render_test(&mut vger, &device, &queue, "arc_stroke_gradient.png", false);
}

#[test]
fn segment_stroke_gradient() {
    let (device, queue) = block_on(setup());

    let mut vger = VGER::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb);

    vger.begin(512.0, 512.0, 1.0, &device);

    let paint = vger.linear_gradient(
        [100.0, 100.0].into(),
        [200.0, 200.0].into(),
        Color::CYAN,
        Color::MAGENTA,
        0.0,
    );

    vger.stroke_segment([100.0, 100.0].into(), [200.0, 200.0].into(), 4.0, paint);

    render_test(
        &mut vger,
        &device,
        &queue,
        "segment_stroke_gradient.png",
        false,
    );
}

#[test]
fn bezier_stroke_gradient() {
    let (device, queue) = block_on(setup());

    let mut vger = VGER::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb);

    vger.begin(512.0, 512.0, 1.0, &device);

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

    render_test(
        &mut vger,
        &device,
        &queue,
        "bezier_stroke_gradient.png",
        false,
    );
}

fn rand2<T: rand::Rng>(rng: &mut T) -> LocalPoint {
    LocalPoint::new(rng.gen_range(0.0, 512.0), rng.gen_range(0.0, 512.0))
}

#[test]
fn path_fill() {
    let (device, queue) = block_on(setup());

    let mut vger = VGER::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb);

    vger.begin(512.0, 512.0, 1.0, &device);

    let paint = vger.linear_gradient(
        [0.0, 0.0].into(),
        [512.0, 512.0].into(),
        Color::CYAN,
        Color::MAGENTA,
        0.0,
    );

    let mut rng = rand::thread_rng();

    let start = rand2(&mut rng);

    vger.move_to(start);

    for _ in 0..10 {
        vger.quad_to(rand2(&mut rng), rand2(&mut rng));
    }

    vger.quad_to(rand2(&mut rng), start);
    vger.fill(paint);

    render_test(&mut vger, &device, &queue, "path_fill.png", true);
}

#[test]
fn text() {
    let (device, queue) = block_on(setup());

    let mut vger = VGER::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb);

    vger.begin(512.0, 512.0, 1.0, &device);

    let paint = vger.linear_gradient(
        [0.0, 0.0].into(),
        [512.0, 512.0].into(),
        Color::CYAN,
        Color::MAGENTA,
        0.0,
    );

    vger.translate([32.0, 256.0].into());
    vger.text("This is a test", 32, None);

    render_test(&mut vger, &device, &queue, "text.png", true);
}

#[test]
fn text_box() {
    let (device, queue) = block_on(setup());

    let mut vger = VGER::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb);

    vger.begin(512.0, 512.0, 1.0, &device);

    let paint = vger.linear_gradient(
        [0.0, 0.0].into(),
        [512.0, 512.0].into(),
        Color::CYAN,
        Color::MAGENTA,
        0.0,
    );

    let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

    vger.translate([32.0, 256.0].into());
    vger.text(lorem, 18, Some(448.0));

    render_test(&mut vger, &device, &queue, "text_box.png", true);
}