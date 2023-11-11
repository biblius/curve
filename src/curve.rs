use std::f32::consts::PI;
use std::time::{Duration, Instant};

use crate::kurve::ArenaBounds;
use crate::point::Line;
use crate::{new_trail_countdown, INV_DURATION, ROT_SPEED, VELOCITY};
use ggez::input::keyboard::KeyCode;
use ggez::mint::Point2;
use ggez::Context;
use rand::Rng;

#[derive(Debug)]
pub struct Curve {
    /// Where the curve is located
    pub position: Point2<f32>,

    /// Rotation angle in rad
    pub rotation: f32,

    /// How fast the curve is moving
    pub velocity: f32,

    /// The movement keycodes for this curve
    pub move_keys: MoveKeys,

    /// The current duration until the trail should be drawn
    pub trail_countdown: Duration,

    /// When the last curve segment started or ended, used in unison with
    /// [trail_active][Self::trail_active]
    pub trail_ts: Instant,

    /// Whether or not this curve should currently draw its trail
    pub trail_active: bool,

    /// Index to the player array, i.e. who this player belongs to
    pub player_id: usize,

    /// The curves for game logic
    pub lines: Vec<Line>,

    pub alive: bool,
}

impl Curve {
    pub fn new_random_pos(player_id: usize, bounds: ArenaBounds, mv_keys: MoveKeys) -> Self {
        let mut rng = rand::thread_rng();
        let p_x: f32 = rng.gen_range(bounds.x_min..bounds.x_max);
        let p_y: f32 = rng.gen_range(bounds.y_min..bounds.y_max);
        let rot: f32 = rng.gen_range(0f32..2. * PI);
        Self {
            position: Point2 { x: p_x, y: p_y },
            rotation: rot,
            velocity: VELOCITY,
            move_keys: mv_keys,
            player_id,
            lines: vec![],

            trail_countdown: new_trail_countdown(),
            trail_ts: std::time::Instant::now(),
            trail_active: true,

            alive: true,
        }
    }

    pub fn new(player_id: usize, pos: Point2<f32>, rot: f32, mv_keys: MoveKeys) -> Self {
        Self {
            position: pos,
            rotation: rot,
            velocity: VELOCITY,
            move_keys: mv_keys,
            player_id,
            lines: vec![],

            trail_countdown: new_trail_countdown(),
            trail_ts: std::time::Instant::now(),
            trail_active: true,

            alive: true,
        }
    }

    /// Checks whether a move key is pressed and rotates the curve accordingly
    #[inline]
    pub fn rotate(&mut self, ctx: &mut Context) {
        if ctx.keyboard.is_key_pressed(self.move_keys.right) {
            self.rotation += ROT_SPEED;
        }

        if ctx.keyboard.is_key_pressed(self.move_keys.left) {
            self.rotation -= ROT_SPEED;
        }
    }

    #[inline]
    pub fn mv(&mut self) {
        self.position.x += self.velocity * self.rotation.cos();
        self.position.y += self.velocity * self.rotation.sin();
    }

    /// Return the curve's next position based on its velocity and rotation
    #[inline]
    pub fn next_pos(&self) -> Point2<f32> {
        Point2 {
            x: self.position.x + self.velocity * self.rotation.cos(),
            y: self.position.y + self.velocity * self.rotation.sin(),
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

    pub fn tick_trail(&mut self) {
        let now = std::time::Instant::now();

        // Disable trail if countdown is done and invulnerability countdown
        if now.duration_since(self.trail_ts) > self.trail_countdown {
            self.trail_active = false;
            self.trail_ts = now;
        }

        // Enable trail if countdown is done
        if now.duration_since(self.trail_ts) > INV_DURATION && !self.trail_active {
            self.trail_active = true;
            self.trail_countdown = new_trail_countdown();
            self.trail_ts = now;
        }

        if self.trail_active {
            // Push the line to the actual self
            let line = Line::interpolate(self.position, self.next_pos());
            self.lines.push(line);
        }
    }
}

#[derive(Debug)]
pub struct MoveKeys {
    pub left: KeyCode,
    pub right: KeyCode,
}
