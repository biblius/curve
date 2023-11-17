use std::{
    f32::consts::{FRAC_PI_8, PI},
    ops::{Index, IndexMut},
};

use ggez::{
    graphics::{Color, InstanceArray, Mesh},
    mint::Point2,
    Context, GameError,
};

use super::curve::Curve;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash)]
pub enum Girth {
    Tiny,
    Small,
    Normal,
    Large,
    Larger,
    Chungus,
}

impl Girth {
    #[inline]
    pub const fn min() -> Self {
        Self::Tiny
    }

    #[inline]
    pub const fn max() -> Self {
        Self::Chungus
    }

    pub fn as_f32(&self) -> f32 {
        match self {
            Girth::Tiny => 1.0,
            Girth::Small => 1.5,
            Girth::Normal => 2.0,
            Girth::Large => 4.0,
            Girth::Larger => 6.0,
            Girth::Chungus => 8.0,
        }
    }

    #[inline]
    pub fn increment(&self) -> Self {
        use Girth as G;
        match self {
            G::Tiny => G::Small,
            G::Small => G::Normal,
            G::Normal => G::Large,
            G::Large => G::Larger,
            G::Larger => G::Chungus,
            G::Chungus => G::Chungus,
        }
    }

    #[inline]
    pub fn decrement(&self) -> Self {
        use Girth as G;
        match self {
            G::Tiny => G::Tiny,
            G::Small => G::Tiny,
            G::Normal => G::Small,
            G::Large => G::Normal,
            G::Larger => G::Large,
            G::Chungus => G::Larger,
        }
    }
}

impl<T> Index<Girth> for [T; 6] {
    type Output = T;

    fn index(&self, index: Girth) -> &Self::Output {
        match index {
            Girth::Tiny => &self[0],
            Girth::Small => &self[1],
            Girth::Normal => &self[2],
            Girth::Large => &self[3],
            Girth::Larger => &self[4],
            Girth::Chungus => &self[5],
        }
    }
}

impl<T> IndexMut<Girth> for [T; 6] {
    fn index_mut(&mut self, index: Girth) -> &mut Self::Output {
        match index {
            Girth::Tiny => &mut self[0],
            Girth::Small => &mut self[1],
            Girth::Normal => &mut self[2],
            Girth::Large => &mut self[3],
            Girth::Larger => &mut self[4],
            Girth::Chungus => &mut self[5],
        }
    }
}

impl Default for Girth {
    fn default() -> Self {
        Self::Normal
    }
}

/// A line obtained from interpolating 2 points.
#[derive(Debug, Clone)]
pub struct Line {
    pub points: Vec<Point2<f32>>,
    pub girth: Girth,
}

impl Line {
    #[inline]
    pub fn interpolate(origin: Point2<f32>, target: Point2<f32>, girth: Girth) -> Self {
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

        Self { points, girth }
    }

    #[inline]
    pub fn line_meshes_and_arrays(
        ctx: &mut Context,
        color: Color,
    ) -> Result<([InstanceArray; 6], [Mesh; 6]), GameError> {
        Ok((
            [
                InstanceArray::new(ctx, None),
                InstanceArray::new(ctx, None),
                InstanceArray::new(ctx, None),
                InstanceArray::new(ctx, None),
                InstanceArray::new(ctx, None),
                InstanceArray::new(ctx, None),
            ],
            [
                Curve::create_mesh(ctx, color, Girth::Tiny)?,
                Curve::create_mesh(ctx, color, Girth::Small)?,
                Curve::create_mesh(ctx, color, Girth::Normal)?,
                Curve::create_mesh(ctx, color, Girth::Large)?,
                Curve::create_mesh(ctx, color, Girth::Larger)?,
                Curve::create_mesh(ctx, color, Girth::Chungus)?,
            ],
        ))
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

#[derive(Debug, Clone)]
pub struct BoundingCircle(pub Vec<Point2<f32>>);

impl BoundingCircle {
    pub fn new(point: Point2<f32>, distance: f32) -> Self {
        let mut points = vec![];
        let mut rot = PI;
        while rot > -PI {
            let point = Point2 {
                x: (point.x + distance * rot.cos()).round(),
                y: (point.y + distance * rot.sin()).round(),
            };
            points.push(point);
            rot -= FRAC_PI_8;
        }

        Self(points)
    }
}

/// Pixels representing a square around a point.
/// The first element is the point itself, the rest are points
/// starting from the right (p.x - distance, p.y - distance) going counter clockwise.
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox([Point2<f32>; 9]);

impl BoundingBox {
    /// Returns a 'box' of rounded positions around the point.
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
