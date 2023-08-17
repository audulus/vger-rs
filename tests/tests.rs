use futures::executor::block_on;
use vger::color::Color;
use vger::defs::*;
use vger::*;
extern crate rand;
mod common;
use common::*;
use std::sync::Arc;

fn setup() -> (Arc<wgpu::Device>, Arc<wgpu::Queue>) {
    let (device, queue) = block_on(common::setup());
    (Arc::new(device), Arc::new(queue))
}

#[test]
fn test_color_hex() {
    let c = Color::hex("#00D4FF").unwrap();
    assert_eq!(c.r, 0.0);
    assert_eq!(c.g, 212.0 / 255.0);
    assert_eq!(c.b, 1.0);
    assert_eq!(c.a, 1.0);

    let c = Color::hex_const("#00D4FF");
    assert_eq!(c.r, 0.0);
    assert_eq!(c.g, 0.831373);
    assert_eq!(c.b, 1.0);
    assert_eq!(c.a, 1.0);

    let c = Color::hex_const("#009BBA");
    assert_eq!(c.r, 0.0);
}

#[test]
fn fill_circle() {
    let (device, queue) = setup();

    let mut vger = Vger::new(
        device.clone(),
        queue.clone(),
        wgpu::TextureFormat::Rgba8UnormSrgb,
    );

    vger.begin(512.0, 512.0, 1.0);
    let cyan = vger.color_paint(Color::CYAN);
    vger.fill_circle([100.0, 100.0], 20.0, cyan);

    render_test(&mut vger, &device, &queue, "circle.png", false);

    assert!(png_not_black("circle.png"));
}

#[test]
fn fill_circle_array() {
    let (device, queue) = setup();

    let mut vger = Vger::new(
        device.clone(),
        queue.clone(),
        wgpu::TextureFormat::Rgba8UnormSrgb,
    );

    vger.begin(512.0, 512.0, 1.0);
    let cyan = vger.color_paint(Color::CYAN);

    for i in 0..5 {
        vger.fill_circle([100.0 * (i as f32), 100.0], 20.0, cyan);
    }

    render_test(&mut vger, &device, &queue, "circle_array.png", false);
}

#[test]
fn fill_circle_translate() {
    let (device, queue) = setup();

    let mut vger = Vger::new(
        device.clone(),
        queue.clone(),
        wgpu::TextureFormat::Rgba8UnormSrgb,
    );

    vger.begin(512.0, 512.0, 1.0);
    let cyan = vger.color_paint(Color::CYAN);
    vger.translate([256.0, 256.0]);
    vger.fill_circle([0.0, 0.0], 20.0, cyan);

    render_test(&mut vger, &device, &queue, "circle_translate.png", false);
}

#[test]
fn fill_rect() {
    let (device, queue) = setup();

    let mut vger = Vger::new(
        device.clone(),
        queue.clone(),
        wgpu::TextureFormat::Rgba8UnormSrgb,
    );

    vger.begin(512.0, 512.0, 1.0);
    let cyan = vger.color_paint(Color::CYAN);
    vger.fill_rect(euclid::rect(100.0, 100.0, 100.0, 100.0), 10.0, cyan);

    render_test(&mut vger, &device, &queue, "rect.png", false);
}

#[test]
fn fill_rect_gradient() {
    let (device, queue) = setup();

    let mut vger = Vger::new(
        device.clone(),
        queue.clone(),
        wgpu::TextureFormat::Rgba8UnormSrgb,
    );

    vger.begin(512.0, 512.0, 1.0);

    let paint = vger.linear_gradient(
        [100.0, 100.0],
        [200.0, 200.0],
        Color::CYAN,
        Color::MAGENTA,
        0.0,
    );

    vger.fill_rect(euclid::rect(100.0, 100.0, 100.0, 100.0), 10.0, paint);

    render_test(&mut vger, &device, &queue, "rect_gradient.png", false);
}

#[test]
fn stroke_rect_gradient() {
    let (device, queue) = setup();

    let mut vger = Vger::new(
        device.clone(),
        queue.clone(),
        wgpu::TextureFormat::Rgba8UnormSrgb,
    );

    vger.begin(512.0, 512.0, 1.0);

    let paint = vger.linear_gradient(
        [100.0, 100.0],
        [200.0, 200.0],
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
    let (device, queue) = setup();

    let mut vger = Vger::new(
        device.clone(),
        queue.clone(),
        wgpu::TextureFormat::Rgba8UnormSrgb,
    );

    vger.begin(512.0, 512.0, 1.0);

    let paint = vger.linear_gradient(
        [100.0, 100.0],
        [300.0, 300.0],
        Color::CYAN,
        Color::MAGENTA,
        0.0,
    );

    vger.stroke_arc(
        [200.0, 200.0],
        100.0,
        4.0,
        0.0,
        std::f32::consts::PI / 2.0,
        paint,
    );

    render_test(&mut vger, &device, &queue, "arc_stroke_gradient.png", false);
}

#[test]
fn segment_stroke_gradient() {
    let (device, queue) = setup();

    let mut vger = Vger::new(
        device.clone(),
        queue.clone(),
        wgpu::TextureFormat::Rgba8UnormSrgb,
    );

    vger.begin(512.0, 512.0, 1.0);

    let paint = vger.linear_gradient(
        [100.0, 100.0],
        [200.0, 200.0],
        Color::CYAN,
        Color::MAGENTA,
        0.0,
    );

    vger.stroke_segment([100.0, 100.0], [200.0, 200.0], 4.0, paint);

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
    let (device, queue) = setup();

    let mut vger = Vger::new(
        device.clone(),
        queue.clone(),
        wgpu::TextureFormat::Rgba8UnormSrgb,
    );

    vger.begin(512.0, 512.0, 1.0);

    let paint = vger.linear_gradient(
        [100.0, 100.0],
        [200.0, 200.0],
        Color::CYAN,
        Color::MAGENTA,
        0.0,
    );

    vger.stroke_bezier([100.0, 100.0], [150.0, 200.0], [200.0, 200.0], 4.0, paint);

    render_test(
        &mut vger,
        &device,
        &queue,
        "bezier_stroke_gradient.png",
        false,
    );
}

fn rand2<T: rand::Rng>(rng: &mut T) -> LocalPoint {
    LocalPoint::new(rng.gen_range(0.0..512.0), rng.gen_range(0.0..512.0))
}

#[test]
fn path_fill() {
    let (device, queue) = setup();

    let mut vger = Vger::new(
        device.clone(),
        queue.clone(),
        wgpu::TextureFormat::Rgba8UnormSrgb,
    );

    vger.begin(512.0, 512.0, 1.0);

    let paint = vger.linear_gradient([0.0, 0.0], [512.0, 512.0], Color::CYAN, Color::MAGENTA, 0.0);

    let mut rng = rand::thread_rng();

    let start = rand2(&mut rng);

    vger.move_to(start);

    for _ in 0..10 {
        vger.quad_to(rand2(&mut rng), rand2(&mut rng));
    }

    vger.quad_to(rand2(&mut rng), start);
    vger.fill(paint);

    let png_name = "path_fill.png";
    render_test(&mut vger, &device, &queue, png_name, true);
    assert!(png_not_black(png_name));
}

#[test]
fn text() {
    let (device, queue) = setup();

    let mut vger = Vger::new(
        device.clone(),
        queue.clone(),
        wgpu::TextureFormat::Rgba8UnormSrgb,
    );

    vger.begin(512.0, 512.0, 1.0);

    vger.translate([32.0, 256.0]);
    vger.text("This is a test", 32, Color::WHITE, None);

    let png_name = "text.png";
    render_test(&mut vger, &device, &queue, png_name, true);
    assert!(png_not_black(png_name));
}

#[test]
fn text_small() {
    let (device, queue) = setup();

    let mut vger = Vger::new(
        device.clone(),
        queue.clone(),
        wgpu::TextureFormat::Rgba8UnormSrgb,
    );

    vger.begin(512.0, 512.0, 1.0);

    vger.translate([32.0, 256.0]);
    vger.text("53", 18, Color::WHITE, None);

    let png_name = "text_small.png";
    render_test(&mut vger, &device, &queue, png_name, true);
    assert!(png_not_black(png_name));

    let atlas_png_name = "text_small_atlas.png";
    save_png(
        &vger.glyph_cache.mask_atlas.atlas_texture,
        &vger::atlas::Atlas::get_texture_desc(),
        &device,
        &queue,
        atlas_png_name,
    );
}

#[test]
fn text_scale() {
    let (device, queue) = setup();

    let mut vger = Vger::new(
        device.clone(),
        queue.clone(),
        wgpu::TextureFormat::Rgba8UnormSrgb,
    );

    vger.begin(256.0, 256.0, 2.0);

    vger.translate([32.0, 128.0]);
    vger.text("This is a test", 32, Color::WHITE, None);

    let png_name = "text_scale.png";
    render_test(&mut vger, &device, &queue, png_name, true);
    assert!(png_not_black(png_name));

    let atlas_png_name = "text_scale_atlas.png";
    save_png(
        &vger.glyph_cache.mask_atlas.atlas_texture,
        &vger::atlas::Atlas::get_texture_desc(),
        &device,
        &queue,
        atlas_png_name,
    );
}

#[test]
fn text_box() {
    let (device, queue) = setup();

    let mut vger = Vger::new(
        device.clone(),
        queue.clone(),
        wgpu::TextureFormat::Rgba8UnormSrgb,
    );

    vger.begin(512.0, 512.0, 1.0);

    let paint = vger.linear_gradient([0.0, 0.0], [512.0, 512.0], Color::CYAN, Color::MAGENTA, 0.0);

    let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

    vger.translate([32.0, 256.0]);

    let bounds = vger.text_bounds(lorem, 18, Some(448.0));

    vger.stroke_rect(bounds.origin, bounds.max(), 10.0, 4.0, paint);

    vger.text(lorem, 18, Color::WHITE, Some(448.0));

    let png_name = "text_box.png";
    render_test(&mut vger, &device, &queue, png_name, true);
    assert!(png_not_black(png_name));

    let atlas_png_name = "text_box_atlas.png";
    save_png(
        &vger.glyph_cache.mask_atlas.atlas_texture,
        &vger::atlas::Atlas::get_texture_desc(),
        &device,
        &queue,
        atlas_png_name,
    );
}

#[test]
fn test_scissor() {
    let (device, queue) = setup();

    let mut vger = Vger::new(
        device.clone(),
        queue.clone(),
        wgpu::TextureFormat::Rgba8UnormSrgb,
    );

    vger.begin(512.0, 512.0, 2.0);

    vger.scissor(euclid::rect(200.0, 200.0, 100.0, 100.0));
    let cyan = vger.color_paint(Color::WHITE);
    vger.fill_rect(euclid::rect(100.0, 100.0, 300.0, 300.0), 10.0, cyan);

    let png_name = "scissor.png";
    render_test(&mut vger, &device, &queue, png_name, true);
    assert!(png_not_black(png_name));
}

#[test]
fn test_scissor_text() {
    let (device, queue) = setup();

    let mut vger = Vger::new(
        device.clone(),
        queue.clone(),
        wgpu::TextureFormat::Rgba8UnormSrgb,
    );

    vger.begin(512.0, 512.0, 1.0);

    let paint = vger.linear_gradient([0.0, 0.0], [512.0, 512.0], Color::CYAN, Color::MAGENTA, 0.0);

    let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

    vger.translate([32.0, 256.0]);
    vger.scissor(euclid::rect(-100.0, -100.0, 400.0, 400.0));

    let bounds = vger.text_bounds(lorem, 18, Some(448.0));

    vger.stroke_rect(bounds.origin, bounds.max(), 10.0, 4.0, paint);

    vger.text(lorem, 18, Color::WHITE, Some(448.0));

    let png_name = "text_box_scissor.png";
    render_test(&mut vger, &device, &queue, png_name, true);
    assert!(png_not_black(png_name));
}

#[test]
fn segment_stroke_stress() {
    let (device, queue) = setup();

    let mut vger = Vger::new(
        device.clone(),
        queue.clone(),
        wgpu::TextureFormat::Rgba8UnormSrgb,
    );

    vger.begin(512.0, 512.0, 1.0);

    let paint = vger.linear_gradient([0.0, 0.0], [512.0, 512.0], Color::CYAN, Color::MAGENTA, 0.0);

    for _ in 0..100000 {
        let mut rng = rand::thread_rng();
        let a = rand2(&mut rng);
        let b = rand2(&mut rng);

        vger.stroke_segment(a, b, 4.0, paint);
    }

    render_test(
        &mut vger,
        &device,
        &queue,
        "segment_stroke_stress.png",
        false,
    );
}

#[test]
fn segment_stroke_vertical() {
    let (device, queue) = setup();

    let mut vger = Vger::new(
        device.clone(),
        queue.clone(),
        wgpu::TextureFormat::Rgba8UnormSrgb,
    );

    vger.begin(512.0, 512.0, 1.0);

    let paint = vger.linear_gradient(
        [100.0, 100.0],
        [100.0, 200.0],
        Color::CYAN,
        Color::MAGENTA,
        0.0,
    );

    vger.stroke_segment([100.0, 100.0], [100.0, 200.0], 4.0, paint);

    render_test(
        &mut vger,
        &device,
        &queue,
        "segment_stroke_vertical.png",
        false,
    );
}

#[test]
fn segment_stroke_horizontal() {
    let (device, queue) = setup();

    let mut vger = Vger::new(
        device.clone(),
        queue.clone(),
        wgpu::TextureFormat::Rgba8UnormSrgb,
    );

    vger.begin(512.0, 512.0, 1.0);

    let paint = vger.linear_gradient(
        [100.0, 100.0],
        [200.0, 100.0],
        Color::CYAN,
        Color::MAGENTA,
        0.0,
    );

    vger.stroke_segment([100.0, 100.0], [200.0, 100.0], 4.0, paint);

    render_test(
        &mut vger,
        &device,
        &queue,
        "segment_stroke_horizontal.png",
        false,
    );
}
