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

    pub fn linear_gradient(start: LocalPoint,
                           end: LocalPoint,
                           inner_color: Color,
                           outer_color: Color,
                           glow: f32) -> Self {

        // Calculate transform aligned to the line
        let mut d = end - start;
        if d.length() < 0.0001 {
            d = LocalVector::new(0.0,1.0);
        }

        let xform = LocalToWorld::new(d.x, d.y, -d.y, d.x, start.x, start.y).inverse().unwrap();

        Self {
            xform,
            inner_color,
            outer_color,
            image: -1,
            glow
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_linear_gradient() {
        let paint = Paint::linear_gradient(
            LocalPoint::new(0.0,0.0),
            LocalPoint::new(1.0,0.0),
            Color::gray(0.0),
            Color::gray(1.0),
            0.0);

        assert_eq!(paint.apply(WorldPoint::new(0.0,0.0)), Color::gray(0.0));
        assert_eq!(paint.apply(WorldPoint::new(0.5,0.0)), Color::gray(0.5));
        assert_eq!(paint.apply(WorldPoint::new(1.0,0.0)), Color::gray(1.0));
    }

}