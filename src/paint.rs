use euclid::*;
use crate::defs::*;

#[derive(Clone, Copy)]
pub struct Paint {
    xform: LocalToWorld,

    inner_color: [f32; 4],
    outer_color: [f32; 4],

    glow: f32,
    image: i32,
}

impl Paint {

    fn apply(&self, p: WorldPoint) -> [f32; 4] {
        [0.0,0.0,0.0,0.0]
    }
}
