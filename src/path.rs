
use std::cmp::Ord;
use euclid::*;
pub struct WorldSpace;
pub type WorldPoint = Point2D<f32, WorldSpace>;

struct Interval {
    a: f32,
    b: f32
}

struct PathSegment {
    cvs: [WorldPoint; 3],
    next: i32,
    previous: i32
}

impl PathSegment {
    pub fn y_interval(&self) -> Interval {
        Interval {
            a: self.cvs[0].y.min(self.cvs[1].y).min(self.cvs[2].y),
            b: self.cvs[0].y.min(self.cvs[1].y).max(self.cvs[2].y)
        }
    }
}

#[derive(PartialEq, PartialOrd)]
struct PathScannerNode {
    coord: f32,
    seg: i32,
    end: bool
}

pub struct PathScanner {
    segments: Vec<PathSegment>,
    nodes: Vec<PathScannerNode>,
    index: i32
}

impl PathScanner {
    pub fn new() -> Self {
        Self {
            segments: vec![],
            nodes: vec![],
            index: 0
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

        self.nodes.clear();
        self.index = 0;

        for i in 0..self.segments.len() {
            let y_interval = self.segments[i].y_interval();
            self.nodes.push(PathScannerNode {
                coord: y_interval.a, seg: i as i32, end: false
            });
            self.nodes.push(PathScannerNode {
                coord: y_interval.b, seg: i as i32, end: true
            });
        }
        
        self.nodes.sort_by(|a, b| a.partial_cmp(b).unwrap());

    }
}