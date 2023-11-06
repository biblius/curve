use std::f32::consts::PI;

use crate::{ROT_SPEED, VELOCITY};
use ggez::input::keyboard::KeyCode;
use ggez::mint::Point2;
use rand::Rng;

#[derive(Debug)]
pub struct Curve {
    /// Where the curve is located
    pub position: Point2<f32>,

    /// Rotation angle in rad
    pub rotation: f32,

    /// How fast the curve is moving
    pub velocity: f32,

    pub move_keys: MoveKeys,
}

impl Curve {
    pub fn new_random_pos(
        x_min: f32,
        x_max: f32,
        y_min: f32,
        y_max: f32,
        mv_keys: MoveKeys,
    ) -> Self {
        let mut rng = rand::thread_rng();
        let p_x: f32 = rng.gen_range(x_min..x_max);
        let p_y: f32 = rng.gen_range(y_min..y_max);
        let rot: f32 = rng.gen_range(0f32..2. * PI);
        Self {
            position: Point2 { x: p_x, y: p_y },
            rotation: rot,
            velocity: VELOCITY,
            move_keys: mv_keys,
        }
    }

    pub fn new(pos: Point2<f32>, rot: f32, mv_keys: MoveKeys) -> Self {
        Self {
            position: pos,
            rotation: rot,
            velocity: VELOCITY,
            move_keys: mv_keys,
        }
    }

    pub fn rotate_left(&mut self) {
        self.rotation -= ROT_SPEED;
    }

    pub fn rotate_right(&mut self) {
        self.rotation += ROT_SPEED;
    }

    pub fn mv(&mut self) {
        self.position.x += self.velocity * self.rotation.cos();
        self.position.y += self.velocity * self.rotation.sin();
    }

    pub fn next_pos(&self) -> Point2<f32> {
        Point2 {
            x: self.position.x + self.velocity * self.rotation.cos(),
            y: self.position.y + self.velocity * self.rotation.sin(),
        }
    }
}

#[derive(Debug)]
pub struct MoveKeys {
    pub left: KeyCode,
    pub right: KeyCode,
}
