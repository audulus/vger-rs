use euclid::*;

pub struct ScreenSpace;
pub type ScreenSize = Size2D<f32, ScreenSpace>;

pub struct WorldSpace;
pub type WorldPoint = Point2D<f32, WorldSpace>;

pub struct LocalSpace {}
pub type LocalPoint = Point2D<f32, LocalSpace>;
pub type LocalVector = Vector2D<f32, LocalSpace>;

pub type LocalToWorld = Transform2D<f32, LocalSpace, WorldSpace>;
pub type WorldToLocal = Transform2D<f32, WorldSpace, LocalSpace>;

pub fn to_mat3x2<A, B>(xform: Transform2D<f32, A, B>) -> [f32; 8] {
    [
        xform.m11, xform.m21, xform.m31, 0.0, xform.m12, xform.m22, xform.m32, 0.0,
    ]
}
