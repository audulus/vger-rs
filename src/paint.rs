use crate::color::*;
use crate::defs::*;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Paint {
    xform: [f32; 8], // mat3x2<f32>

    inner_color: Color, // vec4<f32>
    outer_color: Color, // vec4<f32>

    glow: f32,
    image: i32,

    pad: [i32; 2],
}

impl Paint {
    pub fn apply(&self, p: WorldPoint) -> Color {
        let m = self.xform;
        let xform = WorldToLocal::new(m[0],m[4],m[1],m[5],m[2],m[6]);
        let local_point = xform.transform_point(p);
        let d = local_point
            .clamp(LocalPoint::zero(), LocalPoint::new(1.0, 1.0))
            .x;

        self.inner_color.mix(self.outer_color, d)
    }

    pub fn solid_color(color: Color) -> Self {
        Self {
            xform: [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            inner_color: color,
            outer_color: color,
            image: -1,
            glow: 0.0,
            pad: [0,0]
        }
    }

    pub fn linear_gradient(
        start: LocalPoint,
        end: LocalPoint,
        inner_color: Color,
        outer_color: Color,
        glow: f32,
    ) -> Self {
        // Calculate transform aligned to the line
        let mut d = end - start;
        if d.length() < 0.0001 {
            d = LocalVector::new(0.0, 1.0);
        }

        let xform = LocalToWorld::new(d.x, d.y, -d.y, d.x, start.x, start.y)
            .inverse()
            .unwrap();

        Self {
            xform: to_mat3x2(xform),
            inner_color,
            outer_color,
            image: -1,
            glow,
            pad: [0,0]
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_paint_size() {
        assert_eq!(std::mem::size_of::<Paint>(), 80);
    }

    #[test]
    fn test_linear_gradient() {
        {
            let paint = Paint::linear_gradient(
                LocalPoint::new(0.0, 0.0),
                LocalPoint::new(1.0, 0.0),
                Color::gray(0.0),
                Color::gray(1.0),
                0.0,
            );

            assert_eq!(paint.apply(WorldPoint::new(0.0, 0.0)), Color::gray(0.0));
            assert_eq!(paint.apply(WorldPoint::new(0.5, 0.0)), Color::gray(0.5));
            assert_eq!(paint.apply(WorldPoint::new(1.0, 0.0)), Color::gray(1.0));
        }

        {
            let paint = Paint::linear_gradient(
                LocalPoint::new(0.0, 0.0),
                LocalPoint::new(0.0, 1.0),
                Color::gray(0.0),
                Color::gray(1.0),
                0.0,
            );

            assert_eq!(paint.apply(WorldPoint::new(0.0, 0.0)), Color::gray(0.0));
            assert_eq!(paint.apply(WorldPoint::new(0.0, 1.0)), Color::gray(1.0));
        }

        {
            let paint = Paint::linear_gradient(
                LocalPoint::new(1.0, 0.0),
                LocalPoint::new(2.0, 0.0),
                Color::gray(0.0),
                Color::gray(1.0),
                0.0,
            );

            assert_eq!(paint.apply(WorldPoint::new(0.0, 0.0)), Color::gray(0.0));
            assert_eq!(paint.apply(WorldPoint::new(1.0, 0.0)), Color::gray(0.0));
            assert_eq!(paint.apply(WorldPoint::new(1.5, 0.0)), Color::gray(0.5));
            assert_eq!(paint.apply(WorldPoint::new(2.0, 0.0)), Color::gray(1.0));
            assert_eq!(paint.apply(WorldPoint::new(3.0, 0.0)), Color::gray(1.0));
        }
    }
}
