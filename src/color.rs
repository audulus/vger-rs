
#[derive(Clone, Copy)]
pub struct Color {
    r: f32, g: f32, b: f32, a: f32
}

impl Color {
    pub fn mix(&self, rhs: Color, s: f32) -> Color {
        Color {
            r: (1.0-s) * self.r + s * rhs.r,
            g: (1.0-s) * self.g + s * rhs.g,
            b: (1.0-s) * self.b + s * rhs.b,
            a: (1.0-s) * self.a + s * rhs.a,
        }
    }
}