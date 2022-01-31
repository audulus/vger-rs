use euclid::*;
use crate::defs::*;
use crate::color::*;

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
