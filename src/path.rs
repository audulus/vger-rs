
use euclid::*;
pub struct WorldSpace;
pub type WorldPoint = Point2D<f32, WorldSpace>;

struct PathSegment {
    cvs: [WorldPoint; 3],
    next: i32,
    previous: i32
}

struct PathScannerNode {
    coord: f32,
    seg: i32,
    end: bool
}

pub struct PathScanner {
    segments: Vec<PathSegment>,
    nodes: Vec<PathScannerNode>
}

impl PathScanner {
    pub fn new() -> Self {
        Self {
            segments: vec![],
            nodes: vec![]
        }
    }

    pub fn begin(&mut self, cvs: &[WorldPoint]) {

        self.segments.clear();

        let mut i = 0;
        while i < cvs.len()-2 {
            self.segments.push(PathSegment{
                cvs: [cvs[i], cvs[i+1], cvs[i+2]],
                next: -1,
                previous: -1
            });
            i += 2;
        }

        // Close the path if necessary.
        if let Some(first) = self.segments.first() {
            if let Some(last) = self.segments.last() {
                let start = first.cvs[0];
                let end = last.cvs[2];
                if start != end {
                    self.segments.push(PathSegment{
                        cvs: [end, start.lerp(end, 0.5), start],
                        next: -1,
                        previous: -1
                    })
                }
            }
        }
    }
}