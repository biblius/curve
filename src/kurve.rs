use std::f32::consts::PI;
use std::time::Instant;

use crate::point::Line;
use crate::{curve::Curve, point::BoundingBox, ARENA_H_MOD, ARENA_W_MOD, CURVE_SIZE};
use crate::{new_trail_countdown, VELOCITY, WINNER_GLOAT_DURATION};
use ggez::graphics::{Drawable, PxScale};
use ggez::input::keyboard::KeyCode;
use ggez::{
    graphics::{self, Canvas, Color, DrawParam, InstanceArray},
    mint::Point2,
    Context, GameResult,
};
use rand::distributions::uniform::SampleUniform;
use rand::Rng;

/// Represents the current phase of the game
#[derive(Debug)]
pub enum KurveState {
    /// The game is currently still being prepared
    Preparation,

    /// The game is prepared and waiting to launch
    StartCountdown {
        /// When this phase has started
        started: Instant,
    },

    /// The game is running
    Running,

    /// The game is paused
    Paused,

    /// The game is gloating the winner
    Winner {
        /// When this phase has started
        started: Instant,
    },
}

/// Achtung die main game struct.
#[derive(Debug)]
pub struct Kurve {
    /// Area width and height
    pub size: (f32, f32),

    /// Where the arena starts and ends on each axis
    pub bounds: ArenaBounds,

    /// The curves in the game
    pub curves: Vec<Curve>,

    /// Current game state
    pub state: KurveState,
}

impl Kurve {
    pub fn new(size: (f32, f32), curves: Vec<Curve>, bounds: ArenaBounds) -> Self {
        Self {
            size,
            curves,
            bounds,
            state: KurveState::Running,
        }
    }

    fn toggle_pause(&mut self) {
        match self.state {
            KurveState::Running => self.state = KurveState::Paused,
            KurveState::Paused => self.state = KurveState::Running,
            _ => {}
        }
    }

    fn tick_winner(&mut self, ctx: &mut Context, started: Instant) {
        let now = Instant::now();
        self.move_curves(ctx);
        if now.duration_since(started) >= WINNER_GLOAT_DURATION {
            self.reset_curves();
            self.state = KurveState::StartCountdown {
                started: Instant::now(),
            };
        }
    }

    /// Tick the round countdown
    fn tick_countdown(&mut self, started: Instant) {
        let now = Instant::now();
        if now.duration_since(started) >= WINNER_GLOAT_DURATION {
            self.reset_curves();
            self.state = KurveState::Running;
        }
    }

    /// Reset the curves' positions and liveness
    fn reset_curves(&mut self) {
        for curve in self.curves.iter_mut() {
            curve.position = self.bounds.random_pos();
            curve.alive = true;
            curve.rotation = random_rot();
            curve.lines.clear();
            curve.trail_active = true;
            curve.trail_countdown = new_trail_countdown();
            curve.velocity = VELOCITY;
        }
    }

    /// Process a running game's tick
    fn tick_running(&mut self, ctx: &mut Context) -> Option<usize> {
        let len = self.curves.len();

        // Bitflags for collision
        let mut collisions = 0u8;

        // Calculate collisions
        for (i, curve) in self.curves.iter().enumerate() {
            let bbox = BoundingBox::new(curve.next_pos());

            for (j, curve) in self.curves.iter().enumerate() {
                let lines = &curve.lines;

                // Skip the last line of the current outer curve due to self collision
                let line_count = if i == j {
                    lines.len().saturating_sub(1)
                } else {
                    lines.len()
                };

                for line in lines[0..line_count].iter() {
                    if check_line_collision(bbox, line)
                        || check_border_collision(
                            self.bounds.x_min,
                            self.bounds.x_max,
                            self.bounds.y_min,
                            self.bounds.y_max,
                            bbox,
                        )
                    {
                        collisions |= 1 << i;
                    }
                }
            }
        }

        // Apply collisions
        for i in 0..len {
            let curve = &mut self.curves[i];
            if collisions >> i == 1 {
                curve.velocity = 0.;
                curve.alive = false;
            }
        }

        // Check for winners
        if let Some(winner) = self.check_winner() {
            self.state = KurveState::Winner {
                started: Instant::now(),
            };
            return Some(winner);
        }

        // Process movement
        self.move_curves(ctx);

        None
    }

    fn move_curves(&mut self, ctx: &mut Context) {
        for curve in self.curves.iter_mut() {
            if ctx.keyboard.is_key_pressed(curve.move_keys.right) {
                curve.rotate_right()
            }

            if ctx.keyboard.is_key_pressed(curve.move_keys.left) {
                curve.rotate_left()
            }

            curve.tick_trail();

            curve.mv();
        }
    }

    /// Check whether there is only one curve currently alive
    fn check_winner(&self) -> Option<usize> {
        let mut winner = None;
        let mut alive = 0;

        for curve in self.curves.iter() {
            if curve.alive {
                if alive < 1 {
                    winner = Some(curve.player_id);
                }
                alive += 1;
            }

            if alive > 1 {
                winner = None;
                break;
            }
        }

        winner
    }

    /// Update the game state and return the winner's ID, if any.
    /// The winner's ID indexes into the player vec.
    pub fn update(&mut self, ctx: &mut Context) -> Option<usize> {
        if ctx.keyboard.is_key_pressed(KeyCode::Space) {
            self.toggle_pause();
        }

        if matches!(self.state, KurveState::Paused) {
            return None;
        }

        match self.state {
            KurveState::Running => return self.tick_running(ctx),
            KurveState::Preparation => return None, // TODO,
            KurveState::StartCountdown { started } => self.tick_countdown(started),
            KurveState::Winner { started } => self.tick_winner(ctx, started),
            KurveState::Paused => unreachable!(),
        }

        None
    }

    pub fn draw_countdown(
        &self,
        ctx: &mut Context,
        canvas: &mut Canvas,
        started: Instant,
    ) -> GameResult {
        let (x, y) = ctx.gfx.drawable_size();
        let second = (WINNER_GLOAT_DURATION.saturating_sub(Instant::now().duration_since(started)))
            .as_secs()
            + 1;
        let mut text = graphics::Text::new(second.to_string());
        text.set_scale(PxScale::from(24.));
        let rect = text.dimensions(ctx).unwrap();
        canvas.draw(
            &text,
            DrawParam::default().dest(Point2 {
                x: x * 0.5 - rect.w * 0.5,
                y: y * 0.5,
            }),
        );
        Ok(())
    }

    pub fn draw(&self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
        let (x, y) = ctx.gfx.drawable_size();
        let arena_rect = graphics::Rect::new(
            x * (1. - ARENA_W_MOD) * 0.5,
            y * (1. - ARENA_H_MOD) * 0.5,
            self.size.0,
            self.size.1,
        );

        let arena_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            arena_rect,
            Color::from_rgb(30, 30, 30),
        )?;

        let curve_mesh = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Point2 { x: 0., y: 0. },
            CURVE_SIZE,
            0.1,
            Color::WHITE,
        )?;

        let draw_param = graphics::DrawParam::default();
        canvas.draw(&arena_mesh, draw_param);

        for curve in self.curves.iter() {
            let trail = curve
                .lines
                .iter()
                .fold(InstanceArray::new(ctx, None), |mut acc, el| {
                    for point in el.iter() {
                        acc.push((*point).into());
                    }
                    acc
                });

            canvas.draw_instanced_mesh(curve_mesh.clone(), &trail, DrawParam::default());

            draw_param.dest(curve.position);
            canvas.draw(&curve_mesh, draw_param);

            /*             let c_rect =
                graphics::Rect::new(-CURVE_SIZE, -CURVE_SIZE, CURVE_SIZE * 2., CURVE_SIZE * 2.);
            let c_mesh =
                graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), c_rect, Color::RED)?;

            let bbox = BoundingBox::new(curve.next_pos());
            for bbox in bbox {
                canvas.draw(&c_mesh, draw_param.dest(bbox));
            } */
        }

        if let KurveState::StartCountdown { started } = self.state {
            self.draw_countdown(ctx, canvas, started)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ArenaBounds {
    /// Min x for spawning
    pub x_min: f32,
    /// Max x for spawning
    pub x_max: f32,
    /// Min y for spawning
    pub y_min: f32,
    /// Max y for spawning
    pub y_max: f32,
}

impl ArenaBounds {
    /// Return a random point within this arena's bounds
    pub fn random_pos(&self) -> Point2<f32> {
        random_pos((self.x_min, self.x_max), (self.y_min, self.y_max))
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

#[inline]
pub fn random_pos<T>(bounds_x: (T, T), bounds_y: (T, T)) -> Point2<T>
where
    T: SampleUniform + PartialOrd,
{
    Point2 {
        x: rand::thread_rng().gen_range(bounds_x.0..bounds_x.1),
        y: rand::thread_rng().gen_range(bounds_y.0..bounds_y.1),
    }
}

#[inline]
pub fn random_rot() -> f32 {
    rand::thread_rng().gen_range(0f32..2. * PI)
}
