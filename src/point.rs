use ggez::mint::Point2;

/// A line obtained from interpolating 2 points.
pub struct Line(Vec<Point2<f32>>);

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

        Self(points)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Point2<f32>> {
        self.0.iter()
    }
}

impl IntoIterator for Line {
    type Item = Point2<f32>;

    type IntoIter = std::vec::IntoIter<Point2<f32>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Pixels representing a square around a point.
/// The first element is the point itself, the rest are points
/// starting from the right (p.x + 1) going counter clockwise.
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox([Point2<f32>; 9]);

impl BoundingBox {
    pub fn new(p: Point2<f32>) -> Self {
        Self([
            p,
            Point2 {
                x: (p.x + 1.).round(),
                y: p.y.round(),
            },
            Point2 {
                x: (p.x + 1.).round(),
                y: (p.y - 1.).round(),
            },
            Point2 {
                x: p.x.round(),
                y: (p.y - 1.).round(),
            },
            Point2 {
                x: (p.x - 1.).round(),
                y: (p.y - 1.).round(),
            },
            Point2 {
                x: (p.x - 1.).round(),
                y: p.y.round(),
            },
            Point2 {
                x: (p.x - 1.).round(),
                y: (p.y + 1.).round(),
            },
            Point2 {
                x: p.x.round(),
                y: (p.y + 1.).round(),
            },
            Point2 {
                x: (p.x + 1.).round(),
                y: (p.y + 1.).round(),
            },
        ])
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Point2<f32>> {
        self.0.iter()
    }
}

impl IntoIterator for BoundingBox {
    type Item = Point2<f32>;

    type IntoIter = std::array::IntoIter<Point2<f32>, 9>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
