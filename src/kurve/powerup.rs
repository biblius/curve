use std::time::{Duration, Instant};

use ggez::mint::Point2;

use super::curve::Curve;

/// Modifies the curve in some way
#[derive(Debug)]
pub struct PowerMod {
    pub point: Point2<f32>,
    pub ty: PowerModifier,
}

impl PowerMod {
    pub fn new(point: Point2<f32>, ty: PowerModifier) -> Self {
        Self { point, ty }
    }

    pub fn apply(&self, curve: &mut Curve) {
        match self.ty {
            PowerModifier::SpeedUp => curve.velocity += 10.,
            PowerModifier::RotUp => curve.rotation_speed += 0.005,
            PowerModifier::Invulnerability => {
                curve.trail_active = false;
                curve.trail_ts = Instant::now();
                curve.trail_countdown = Duration::MAX;
            }
            PowerModifier::Anorexia => curve.girth -= 0.2,
            PowerModifier::SpeedDown => curve.velocity -= 10.,
            PowerModifier::RotDown => curve.rotation_speed -= 0.005,
            // PowerModifier::RightAngle => todo!(),
            PowerModifier::Chungus => curve.girth += 0.2,
        }
    }
}

/// Reverses any modification caused by a powerup
#[derive(Debug)]
pub struct PowerTimeout {
    started: Instant,
    ty: PowerModifier,
}

impl PowerTimeout {
    pub fn apply(&self, curve: &mut Curve) {
        match self.ty {
            PowerModifier::SpeedUp => {
                if curve.velocity > 10. {
                    curve.velocity -= 10.
                }
            }
            PowerModifier::RotUp => curve.rotation_speed -= 0.005,
            PowerModifier::Invulnerability => {
                curve.trail_active = true;
                curve.trail_ts = Instant::now();
                curve.trail_countdown = Curve::new_trail_countdown();
            }
            PowerModifier::Anorexia => curve.girth += 0.2,
            PowerModifier::SpeedDown => curve.velocity += 10.,
            PowerModifier::RotDown => curve.rotation_speed += 0.005,
            // PowerModifier::RightAngle => todo!(),
            PowerModifier::Chungus => curve.girth -= 0.2,
        }
    }
}

/// All possible variations for a power up/down.
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
