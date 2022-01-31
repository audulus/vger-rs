use euclid::*;
use crate::defs::*;

#[derive(Clone, Copy)]
struct Color {
    r: f32, g: f32, b: f32, a: f32
}

impl Color {
    fn mix(&self, rhs: Color, s: f32) -> Color {
        Color {
            r: (1.0-s) * self.r + s * rhs.r,
            g: (1.0-s) * self.g + s * rhs.g,
            b: (1.0-s) * self.b + s * rhs.b,
            a: (1.0-s) * self.a + s * rhs.a,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Paint {
    xform: WorldToLocal,

    inner_color: Color,
    outer_color: Color,

    glow: f32,
    image: i32,
}

impl Paint {

    pub fn apply(&self, p: WorldPoint) -> Color {
        let local_point = self.xform.transform_point(p);
        let d = local_point.clamp(LocalPoint::zero(), LocalPoint::new(1.0,1.0)).x;

        self.inner_color.mix(self.outer_color, d)
    }
}
