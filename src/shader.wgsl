
fn proj(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return normalize(a) * dot(a,b) / length(a);
}

//inline float2 orth(float2 a, float2 b) {
//    return b - proj(a, b);
//}

//inline float2 rot90(float2 p) {
//    return {-p.y, p.x};
//}

fn vs_main() { }