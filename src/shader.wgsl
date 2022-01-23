
fn proj(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return normalize(a) * dot(a,b) / length(a);
}

fn orth(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return b - proj(a, b);
}

fn rot90(p: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(-p.y, p.x);
}

fn vs_main() { }