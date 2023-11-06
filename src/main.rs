use curve::{Curve, MoveKeys};
use ggez::event::{self};
use ggez::graphics::{DrawParam, InstanceArray};
use ggez::input::keyboard::KeyCode;
use ggez::mint::Point2;
use ggez::{graphics, graphics::Color};
use ggez::{Context, GameResult};
use point::{BoundingBox, Line};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_8};

mod curve;
mod game;
mod point;

const GRID_SIZE: f32 = 3.;
const CURVE_SIZE: f32 = GRID_SIZE * 0.5;

const ROT_SPEED: f32 = FRAC_PI_8 * 0.1;
const VELOCITY: f32 = 1.;

/// Achtung die main game struct.
struct Kurve {
    /// Area width
    size_x: f32,

    /// Area height
    size_y: f32,

    /// The curves in the game
    curves: Vec<Curve>,

    /// A vec containing a flattened repr of all the lines' points.
    /// Used to draw the curves.
    trails: Vec<InstanceArray>,

    /// The curves for game logic
    lines: Vec<Line>,
}

impl Kurve {
    pub fn new(size_x: f32, size_y: f32, curves: Vec<Curve>, trails: Vec<InstanceArray>) -> Self {
        Kurve {
            size_x,
            size_y,
            curves,
            trails,
            lines: Vec::new(),
        }
    }

    #[inline]
    pub fn center(&self) -> Point2<f32> {
        Point2 {
            x: self.size_x * 0.5,
            y: self.size_y * 0.5,
        }
    }

    pub fn update(&mut self, ctx: &mut Context) {
        let mut new_lines = 0;
        for i in 0..self.curves.len() {
            // Append the trail
            let curve = &mut self.curves[i];
            let trail = &mut self.trails[i];

            if ctx.keyboard.is_key_pressed(curve.move_keys.right) {
                curve.rotate_right()
            }

            if ctx.keyboard.is_key_pressed(curve.move_keys.left) {
                curve.rotate_left()
            }

            let line = Line::interpolate(curve.position, curve.next_pos());
            for p in line.iter() {
                trail.push((*p).into());
            }

            self.lines.push(line);
            new_lines += 1;

            curve.mv();
        }

        // Skip the newly added lines.
        for line in self.lines[0..self.lines.len() - new_lines].iter() {
            for curve in self.curves.iter_mut() {
                let bbox = BoundingBox::new(curve.next_pos());
                if check_line_collision(bbox, line)
                    || check_border_collision(self.size_x, self.size_y, bbox)
                {
                    curve.velocity = 0.;
                }
            }
        }
    }
}

struct GameState {
    kurve: Kurve,
}

impl GameState {
    pub fn new(ctx: &mut Context) -> Self {
        let (size_x, size_y) = ctx.gfx.drawable_size();
        let center = Point2 {
            x: size_x * 0.5,
            y: size_y * 0.5,
        };

        let player1 = Curve::new(
            Point2 {
                x: center.x + 100.,
                y: center.y,
            },
            0.,
            MoveKeys {
                left: KeyCode::Q,
                right: KeyCode::W,
            },
        );

        let player2 = Curve::new(
            Point2 {
                x: center.x - 100.,
                y: center.y,
            },
            FRAC_PI_2,
            MoveKeys {
                left: KeyCode::Left,
                right: KeyCode::Down,
            },
        );

        //Arena
        let kurve = Kurve::new(
            size_x,
            size_y,
            vec![player1, player2],
            vec![InstanceArray::new(ctx, None), InstanceArray::new(ctx, None)],
        );

        Self { kurve }
    }
}

#[inline]
pub fn check_line_collision(bbox: BoundingBox, line: &Line) -> bool {
    for bp in bbox.iter() {
        for pt in line.iter() {
            if pt.x == bp.x && pt.y == bp.y {
                return true;
            }
        }
    }
    false
}

#[inline]
pub fn check_border_collision(x_max: f32, y_max: f32, bbox: BoundingBox) -> bool {
    for point in bbox {
        if point.x < 0. || point.x > x_max || point.y < 0. || point.y > y_max {
            return true;
        }
    }
    false
}

impl event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.kurve.update(ctx);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, None);

        let arena_rect = graphics::Rect::new(
            -self.kurve.size_x * 0.5,
            -self.kurve.size_y * 0.5,
            self.kurve.size_x,
            self.kurve.size_y,
        );

        let arena_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            arena_rect,
            Color::BLUE,
        )?;

        let curve_mesh = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Point2 { x: 0., y: 0. },
            CURVE_SIZE,
            0.1,
            Color::WHITE,
        )?;

        let draw_param = graphics::DrawParam::default().dest(self.kurve.center());
        canvas.draw(&arena_mesh, draw_param);

        for (i, curve) in self.kurve.curves.iter().enumerate() {
            let trail = &self.kurve.trails[i];
            canvas.draw_instanced_mesh(curve_mesh.clone(), trail, DrawParam::default());

            draw_param.dest(curve.position);
            canvas.draw(&curve_mesh, draw_param);

            let c_rect =
                graphics::Rect::new(-CURVE_SIZE, -CURVE_SIZE, CURVE_SIZE * 2., CURVE_SIZE * 2.);
            let c_mesh =
                graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), c_rect, Color::RED)?;

            let bbox = BoundingBox::new(curve.next_pos());
            for bbox in bbox {
                canvas.draw(&c_mesh, draw_param.dest(bbox));
            }
        }

        canvas.finish(ctx)?;

        Ok(())
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("curve", "biblius");
    let (mut ctx, event_loop) = cb.build()?;
    ctx.gfx.set_window_title("curve");
    for res in ctx.gfx.supported_resolutions() {
        dbg!(res);
    }
    dbg!(ctx.gfx.drawable_size());

    let state = GameState::new(&mut ctx);
    event::run(ctx, event_loop, state);
}
