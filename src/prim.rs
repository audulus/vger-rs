use euclid::*;

#[derive(Copy, Clone)]
pub enum PrimType {

    /// Filled circle.
    Circle, 

    /// Stroked arc.
    Arc,

    /// Rounded corner rectangle.
    Rect,  

    /// Stroked rounded rectangle.
    RectStroke,

    /// Single-segment quadratic bezier curve.
    Bezier,

    /// line segment
    Segment,

    /// Multi-segment bezier curve.
    Curve,

    /// Connection wire. See https://www.shadertoy.com/view/NdsXRl
    Wire,
    
    /// Text rendering.
    Glyph,

    /// Path fills.
    PathFill
}

impl Default for PrimType {
    fn default() -> PrimType {
        PrimType::Circle
    }
}

pub struct LocalSpace {}
pub type LocalPoint = Point2D<f32, LocalSpace>;

#[derive(Copy, Clone, Default)]
pub struct Prim {
    /// Type of primitive.
    pub prim_type: PrimType,

    /// Stroke width.
    pub width: f32,

    /// Radius of circles. Corner radius for rounded rectangles.
    pub radius: f32,

    /// Control vertices.
    pub cvs: [LocalPoint; 3],

    /// Start of the control vertices, if they're in a separate buffer.
    pub start: u32,

    /// Number of control vertices (vgerCurve and vgerPathFill)
    pub count: u32,

    /// Index of paint applied to drawing region.
    pub paint: u32,

    /// Glyph region index. (used internally)
    pub glyph: u32,

    /// Index of transform applied to drawing region. (used internally)
    pub xform: u32,

    /// Min and max coordinates of the quad we're rendering. (used internally)
    pub quad_bounds: [f32; 4],

    /// Min and max coordinates in texture space. (used internally)
    pub tex_bounds: [f32; 4],
}
