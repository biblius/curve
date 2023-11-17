use std::{
    fmt::Display,
    time::{Duration, Instant},
};

use ggez::{mint::Point2, Context, GameResult};

use super::{curve::Curve, point::Girth, POWERMOD_SIZE};

/// Modifies the curve in some way
#[derive(Debug)]
pub struct PowerMod {
    pub point: Point2<f32>,
    pub ty: PowerModifier,
}

impl PowerMod {
    #[inline]
    pub fn new(point: Point2<f32>, ty: PowerModifier) -> Self {
        Self { point, ty }
    }

    #[inline]
    pub fn bounds(&self) -> [f32; 4] {
        [
            self.point.x - POWERMOD_SIZE * 0.5,
            self.point.x + POWERMOD_SIZE * 0.5,
            self.point.y - POWERMOD_SIZE * 0.5,
            self.point.y + POWERMOD_SIZE * 0.5,
        ]
    }
}

/// Reverses any modification caused by a powerup
#[derive(Debug)]
pub struct PowerTimeout {
    pub curve: usize,
    pub started: Instant,
    pub ty: PowerModifier,
}

/// All possible variations for a power up/down.
#[repr(usize)]
#[derive(Debug, Clone, Copy)]
pub enum PowerModifier {
    // Good
    /// Increases velocity
    SpeedUp,

    /// Increases rotation speed
    RotUp,

    /// Makes the curve invulnerable
    Invulnerability,

    /// Makes the curve thinner
    Anorexia,

    // Bad
    /// Decreases player velocity
    SpeedDown,

    // Decreases rotation speed
    RotDown,

    // Constricts rotation to right angles
    // RightAngle,
    /// Makes the curve fatter
    Chungus,
}

const ROTUP: f32 = 0.01;
const VELO: f32 = 10.;

impl PowerModifier {
    pub fn apply(&self, _ctx: &mut Context, curve: &mut Curve) -> GameResult {
        match self {
            PowerModifier::SpeedUp => curve.velocity += VELO,
            PowerModifier::RotUp => curve.rotation_speed += ROTUP,
            PowerModifier::Invulnerability => {
                curve.trail_active = false;
                curve.trail_ts = Instant::now();
                curve.trail_fuse = Duration::MAX;
            }
            PowerModifier::SpeedDown => curve.velocity -= VELO,
            PowerModifier::RotDown => curve.rotation_speed -= ROTUP,
            // PowerModifier::RightAngle => todo!(),
            PowerModifier::Anorexia => {
                if curve.girth > Girth::min() {
                    curve.girth = curve.girth.decrement();
                }
            }
            PowerModifier::Chungus => {
                if curve.girth < Girth::max() {
                    curve.girth = curve.girth.increment();
                }
            }
        }
        Ok(())
    }

    pub fn remove(&self, _ctx: &mut Context, curve: &mut Curve) -> GameResult {
        match self {
            PowerModifier::SpeedUp => {
                if curve.velocity > VELO {
                    curve.velocity -= VELO
                }
            }
            PowerModifier::RotUp => curve.rotation_speed -= ROTUP,
            PowerModifier::Invulnerability => {
                curve.trail_active = true;
                curve.trail_ts = Instant::now();
                curve.trail_fuse = Curve::new_trail_fuse();
            }
            PowerModifier::Anorexia => {
                curve.girth = curve.girth.increment();
            }
            PowerModifier::SpeedDown => curve.velocity += VELO,
            PowerModifier::RotDown => curve.rotation_speed += ROTUP,
            // PowerModifier::RightAngle => todo!(),
            PowerModifier::Chungus => {
                curve.girth = curve.girth.decrement();
            }
        }
        Ok(())
    }
}

impl Display for PowerModifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PowerModifier::SpeedUp => write!(f, "SpeedUp"),
            PowerModifier::RotUp => write!(f, "RotUp"),
            PowerModifier::Invulnerability => write!(f, "Invul"),
            PowerModifier::Anorexia => write!(f, "Thin"),
            PowerModifier::SpeedDown => write!(f, "SpeedDown"),
            PowerModifier::RotDown => write!(f, "RotDown"),
            PowerModifier::Chungus => write!(f, "Fat"),
        }
    }
}
