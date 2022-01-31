
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32
}

impl Color {

    pub fn gray(lightness: f32) -> Color {
        Self {
            r: lightness, g: lightness, b: lightness, a: 1.0
        }
    }

    pub fn mix(&self, rhs: Color, s: f32) -> Color {
        Color {
            r: (1.0-s) * self.r + s * rhs.r,
            g: (1.0-s) * self.g + s * rhs.g,
            b: (1.0-s) * self.b + s * rhs.b,
            a: (1.0-s) * self.a + s * rhs.a,
        }
    }
}