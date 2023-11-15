use ggez::mint::Point2;

/// A line obtained from interpolating 2 points.
#[derive(Debug, Clone)]
pub struct Line {
    points: Vec<Point2<f32>>,
}

impl Line {
    #[inline]
    pub fn interpolate(origin: Point2<f32>, target: Point2<f32>) -> Self {
        let mut points = vec![];
        let d_x = target.x - origin.x;
        let d_y = target.y - origin.y;
        let max = d_x.abs().max(d_y.abs()).max(1.);

        let step_x = d_x / max;
        let step_y = d_y / max;
        let mut i = 0.;
        while i < max {
            let pos_x = origin.x + i * step_x;
            let pos_y = origin.y + i * step_y;
            points.push(Point2 {
                x: pos_x.round(),
                y: pos_y.round(),
            });
            i += 1.;
        }

        Self { points }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Point2<f32>> {
        self.points.iter()
    }
}

impl IntoIterator for Line {
    type Item = Point2<f32>;

    type IntoIter = std::vec::IntoIter<Point2<f32>>;

    fn into_iter(self) -> Self::IntoIter {
        self.points.into_iter()
    }
}

/// Pixels representing a square around a point.
/// The first element is the point itself, the rest are points
/// starting from the right (p.x + 1) going counter clockwise.
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox([Point2<f32>; 9]);

impl BoundingBox {
    /// Returns a 'box' of rounded positions around the point with a distance of 1.
    pub fn new(p: Point2<f32>, distance: f32) -> Self {
        // All points are clockwise (save the first) for
        // polygon drawing
        Self([
            Point2 {
                x: p.x.round(),
                y: p.y.round(),
            },
            Point2 {
                x: (p.x - distance).round(),
                y: (p.y - distance).round(),
            },
            Point2 {
                x: p.x.round(),
                y: (p.y - distance).round(),
            },
            Point2 {
                x: (p.x + distance).round(),
                y: (p.y - distance).round(),
            },
            Point2 {
                x: (p.x + distance).round(),
                y: p.y.round(),
            },
            Point2 {
                x: (p.x + distance).round(),
                y: (p.y + distance).round(),
            },
            Point2 {
                x: p.x.round(),
                y: (p.y + distance).round(),
            },
            Point2 {
                x: (p.x - distance).round(),
                y: (p.y + distance).round(),
            },
            Point2 {
                x: (p.x - distance).round(),
                y: p.y.round(),
            },
        ])
    }

    /*     pub fn expand(&mut self, amount: f32) {
        self.0[1].x -= amount;
        self.0[1].y -= amount;
        self.0[2].y -= amount;
        self.0[3].x += amount;
        self.0[3].y -= amount;
        self.0[4].x += amount;
        self.0[5].x += amount;
        self.0[5].y += amount;
        self.0[6].y += amount;
        self.0[7].x -= amount;
        self.0[7].y += amount;
        self.0[8].x -= amount;
    } */

    /// Return the bounding box as polygon points for drawing (without the center point)
    pub fn as_polygon(&self) -> &[Point2<f32>] {
        &self.0[1..]
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Point2<f32>> {
        self.0.iter()
    }

    pub fn xs(&self) -> [f32; 9] {
        self.0.map(|p| p.x)
    }

    pub fn ys(&self) -> [f32; 9] {
        self.0.map(|p| p.y)
    }
}

impl IntoIterator for BoundingBox {
    type Item = Point2<f32>;

    type IntoIter = std::array::IntoIter<Point2<f32>, 9>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
