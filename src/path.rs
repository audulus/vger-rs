
use euclid::*;
pub struct WorldSpace;
pub type WorldPoint = Point2D<f32, WorldSpace>;

pub struct PathSegment {
    cvs: [WorldPoint; 3],
    next: i32,
    previous: i32
}

pub struct PathScanner {
    segments: Vec<PathSegment>
}

impl PathScanner {
    pub fn new() -> Self {
        Self {
            segments: vec![]
        }
    }
}