use std::collections::VecDeque;
use std::f32::consts::PI;
use std::fmt::{Debug, Display};
use std::time::{Duration, Instant};

use super::point::Line;
use super::{DEFAULT_GIRTH, DEFAULT_ROTATION, INV_DURATION, TRAIL_SKIP_MAX, TRAIL_SKIP_MIN};
use crate::display_key;
use crate::kurve::ArenaBounds;
use ggez::graphics::Color;
use ggez::input::keyboard::KeyCode;
use ggez::mint::Point2;
use ggez::{graphics, Context, GameError};
use rand::Rng;

pub struct Curve {
    /// Index to the player array, i.e. who this curve belongs to
    pub player_id: usize,

    /// Where the curve is located
    pub position: Point2<f32>,

    /// Rotation angle in rad
    pub rotation: f32,

    /// How fast the curve is moving
    pub velocity: f32,

    /// How much to increment rotation on movement
    pub rotation_speed: f32,

    /// Used for multiplying the bounding box distance
    pub girth: f32,

    /// The movement keycodes for this curve
    pub move_keys: MoveKeys,

    /// The current duration until the trail should be drawn
    pub trail_countdown: Duration,

    /// When the last curve segment started or ended, used in unison with
    /// [trail_active][Self::trail_active]
    pub trail_ts: Instant,

    /// Whether or not this curve should currently draw its trail
    pub trail_active: bool,

    /// The curves for game logic
    pub lines: VecDeque<Line>,

    pub alive: bool,

    pub mesh: graphics::Mesh,

    pub color: Color,
}

impl Debug for Curve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Curve")
            .field("player_id", &self.player_id)
            .field("position", &self.position)
            .field("rotation", &self.rotation)
            .field("velocity", &self.velocity)
            .field("move_keys", &self.move_keys)
            .field("trail_countdown", &self.trail_countdown)
            .field("trail_ts", &self.trail_ts)
            .field("trail_active", &self.trail_active)
            .field("alive", &self.alive)
            .field("color", &self.color)
            .finish()
    }
}

impl Curve {
    pub fn new_random_pos(
        ctx: &mut Context,
        player_id: usize,
        bounds: ArenaBounds,
        mv_keys: MoveKeys,
        color: Color,
        alive: bool,
        velocity: f32,
    ) -> Result<Self, GameError> {
        let mut rng = rand::thread_rng();
        let p_x: f32 = rng.gen_range(bounds.x_min..bounds.x_max);
        let p_y: f32 = rng.gen_range(bounds.y_min..bounds.y_max);
        let rot: f32 = rng.gen_range(0f32..2. * PI);

        Ok(Self {
            position: Point2 { x: p_x, y: p_y },
            rotation: rot,
            velocity,
            rotation_speed: DEFAULT_ROTATION,
            girth: DEFAULT_GIRTH,

            move_keys: mv_keys,
            player_id,
            lines: VecDeque::new(),

            trail_countdown: Self::new_trail_countdown(),
            trail_ts: std::time::Instant::now(),
            trail_active: true,

            alive,

            mesh: Self::create_mesh(ctx, color)?,
            color,
        })
    }

    /*     pub fn new(player_id: usize, pos: Point2<f32>, rot: f32, mv_keys: MoveKeys) -> Self {
        Self {
            position: pos,
            rotation: rot,
            velocity: VELOCITY,
            move_keys: mv_keys,
            player_id,
            lines: VecDeque::new(),

            trail_countdown: new_trail_countdown(),
            trail_ts: std::time::Instant::now(),
            trail_active: true,

            alive: true,
        }
    } */

    /// Checks whether a move key is pressed and rotates the curve accordingly
    #[inline]
    pub fn rotate(&mut self, ctx: &mut Context) {
        if ctx.keyboard.is_key_pressed(self.move_keys.cw) {
            self.rotation += self.rotation_speed;
        }

        if ctx.keyboard.is_key_pressed(self.move_keys.ccw) {
            self.rotation -= self.rotation_speed;
        }
    }

    #[inline]
    pub fn mv(&mut self, delta: f32) {
        self.position.x += self.velocity * delta * self.rotation.cos();
        self.position.y += self.velocity * delta * self.rotation.sin();
    }

    /// Return the curve's next position based on its velocity and rotation
    #[inline]
    pub fn next_pos(&self, delta: f32) -> Point2<f32> {
        Point2 {
            x: self.position.x + self.velocity * delta * self.rotation.cos(),
            y: self.position.y + self.velocity * delta * self.rotation.sin(),
        }
    }

    /// The same as `next_pos`, except uses a larger multiplier instead of velocity
    /// to get the point to draw the line to during countdown
    #[inline]
    pub fn project_rotation(&self) -> Point2<f32> {
        Point2 {
            x: self.position.x + 20. * self.rotation.cos(),
            y: self.position.y + 20. * self.rotation.sin(),
        }
    }

    /// Process the curve's trail and append a line to its lines if the trail is active
    pub fn tick_trail(&mut self, delta: f32) {
        let now = std::time::Instant::now();

        // Disable trail if countdown is done and invulnerability countdown
        if now.duration_since(self.trail_ts) > self.trail_countdown {
            self.trail_active = false;
            self.trail_ts = now;
        }

        // Enable trail if countdown is done
        if now.duration_since(self.trail_ts) > INV_DURATION && !self.trail_active {
            self.trail_active = true;
            self.trail_countdown = Self::new_trail_countdown();
            self.trail_ts = now;
        }

        if self.trail_active {
            // Push the line to the actual self
            let line = Line::interpolate(self.position, self.next_pos(delta));
            self.lines.push_back(line);
        }
    }

    #[inline]
    pub fn create_mesh(ctx: &mut Context, color: Color) -> Result<graphics::Mesh, GameError> {
        graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Point2 { x: 0., y: 0. },
            2.,
            0.1,
            color,
        )
    }

    /// Get a random duration for counting down the segment skip in the curves
    #[inline]
    pub fn new_trail_countdown() -> Duration {
        let mut rng = rand::thread_rng();
        let millis = rng.gen_range(TRAIL_SKIP_MIN..TRAIL_SKIP_MAX);
        Duration::from_millis(millis)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MoveKeys {
    pub cw: KeyCode,
    pub ccw: KeyCode,
}

impl Default for MoveKeys {
    fn default() -> Self {
        Self {
            cw: KeyCode::Q,
            ccw: KeyCode::W,
        }
    }
}

impl Display for MoveKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (l, r) = (
            display_key(self.ccw).unwrap_or("???"),
            display_key(self.cw).unwrap_or("???"),
        );
        write!(f, "{l}/{r}")
    }
}
