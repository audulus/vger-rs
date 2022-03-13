#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn gray(lightness: f32) -> Color {
        Self {
            r: lightness,
            g: lightness,
            b: lightness,
            a: 1.0,
        }
    }

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn mix(&self, rhs: Color, s: f32) -> Color {
        Color {
            r: (1.0 - s) * self.r + s * rhs.r,
            g: (1.0 - s) * self.g + s * rhs.g,
            b: (1.0 - s) * self.b + s * rhs.b,
            a: (1.0 - s) * self.a + s * rhs.a,
        }
    }

    pub fn alpha(&self, a: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }

    pub const CYAN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const MAGENTA: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
}
