use curve::{Curve, MoveKeys};
use ggez::conf::WindowMode;
use ggez::event::{self};
use ggez::graphics;
use ggez::input::keyboard::KeyCode;
use ggez::mint::Point2;
use ggez::winit::dpi::PhysicalSize;
use ggez::{Context, GameResult};
use kurve::{ArenaBounds, Kurve};
use point::{BoundingBox, Line};
use rand::Rng;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_8};
use std::time::Duration;

mod curve;
mod kurve;
mod point;

const GRID_SIZE: f32 = 3.;
const CURVE_SIZE: f32 = GRID_SIZE * 0.5;

const ROT_SPEED: f32 = FRAC_PI_8 * 0.1;
const VELOCITY: f32 = 1.;

const ARENA_W_MOD: f32 = 0.8;
const ARENA_H_MOD: f32 = 0.8;

const TRAIL_INTERVAL_MIN: u64 = 2000;
const TRAIL_INTERVAL_MAX: u64 = 4000;

/// Curve invulnerability duration when it is not leaving the trail
const INV_DURATION: Duration = Duration::from_millis(300);

/// Get a random duration for counting down the segment skip in the curves
pub fn new_trail_countdown() -> Duration {
    let mut rng = rand::thread_rng();
    let millis = rng.gen_range(TRAIL_INTERVAL_MIN..TRAIL_INTERVAL_MAX);
    Duration::from_millis(millis)
}

struct Game {
    kurve: Kurve,

    last_window_size: PhysicalSize<u32>,
}

impl Game {
    pub fn new(ctx: &mut Context) -> Self {
        let (size_x, size_y) = ctx.gfx.drawable_size();

        /*         let player1 = Curve::new(
                   1,
                   Point2 {
                       x: 400. + 100.,
                       y: 300.,
                   },
                   0.,
                   MoveKeys {
                       left: KeyCode::Q,
                       right: KeyCode::W,
                   },
               );

               let player2 = Curve::new(
                   2,
                   Point2 {
                       x: 400. - 100.,
                       y: 300.,
                   },
                   FRAC_PI_2,
                   MoveKeys {
                       left: KeyCode::Left,
                       right: KeyCode::Down,
                   },
               );
        */

        let size = (size_x * ARENA_W_MOD, size_y * ARENA_H_MOD);
        let center = Point2 {
            x: size_x * 0.5,
            y: size_y * 0.5,
        };

        let (x_min, x_max) = (center.x - size.0 * 0.5, center.x + size.0 * 0.5);
        let (y_min, y_max) = (center.y - size.1 * 0.5, center.y + size.1 * 0.5);
        let bounds = ArenaBounds {
            x_min,
            x_max,
            y_min,
            y_max,
        };

        let player1 = Curve::new_random_pos(
            0,
            bounds,
            MoveKeys {
                left: KeyCode::Q,
                right: KeyCode::W,
            },
        );

        let player2 = Curve::new_random_pos(
            1,
            bounds,
            MoveKeys {
                left: KeyCode::Left,
                right: KeyCode::Down,
            },
        );

        //Arena
        let kurve = Kurve::new(size, center, vec![player1, player2], bounds);

        Self {
            kurve,
            last_window_size: ctx.gfx.window().inner_size(),
        }
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
pub fn check_border_collision(
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
    bbox: BoundingBox,
) -> bool {
    for point in bbox {
        if point.x < x_min || point.x > x_max || point.y < y_min || point.y > y_max {
            return true;
        }
    }

    false
}

impl event::EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if self.last_window_size != ctx.gfx.window().inner_size() {
            let size = ctx.gfx.drawable_size();
            self.kurve.resize(size);
        }
        self.kurve.update(ctx);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, None);

        self.kurve.draw(ctx, &mut canvas)?;

        canvas.finish(ctx)?;

        Ok(())
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("curve", "biblius");
    let (mut ctx, event_loop) = cb.build()?;
    ctx.gfx.set_window_title("curve");
    /*     let res = ctx
        .gfx
        .supported_resolutions()
        .next()
        .unwrap()
        .to_logical(1.);
    ctx.gfx
        .set_mode(
            WindowMode::default()
                .dimensions(res.width, res.height)
                .resizable(true),
        )
        .unwrap();
    ctx.gfx.set_drawable_size(res.width, res.height).unwrap(); */

    let state = Game::new(&mut ctx);
    event::run(ctx, event_loop, state);
}
