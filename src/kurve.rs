use std::collections::HashMap;

use crate::{
    check_border_collision, check_line_collision,
    curve::Curve,
    new_trail_countdown,
    point::{BoundingBox, Line},
    ARENA_H_MOD, ARENA_W_MOD, CURVE_SIZE, INV_DURATION,
};
use ggez::{
    graphics::{self, Canvas, Color, DrawParam, InstanceArray},
    mint::Point2,
    Context, GameResult,
};

#[derive(Debug)]
pub struct Player {
    score: u8,
}

/// Achtung die main game struct.
pub struct Kurve {
    /// Area width and height
    pub size: (f32, f32),

    /// Where the arena starts and ends on each axis
    pub bounds: ArenaBounds,

    /// The curves in the game
    pub curves: Vec<Curve>,

    pub players: HashMap<u8, Player>,
}

impl Kurve {
    pub fn new(
        size: (f32, f32),
        center: Point2<f32>,
        curves: Vec<Curve>,
        bounds: ArenaBounds,
    ) -> Self {
        Self {
            size,
            curves,
            players: HashMap::new(),
            bounds,
        }
    }

    #[inline]
    pub fn resize(&mut self, size: (f32, f32)) {
        self.size = size;
    }

    pub fn update(&mut self, ctx: &mut Context) {
        // Skip the newly added lines.
        let len = self.curves.len();

        // Bitflags for collision
        let mut collisions = 0u8;

        for i in 0..len {
            let curve = &self.curves[i];
            let bbox = BoundingBox::new(curve.next_pos());

            for j in 0..len {
                let lines = &self.curves[j].lines;

                // Skip the last line of the curve in question due to self collision
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

        for i in 0..len {
            let curve = &mut self.curves[i];
            if collisions >> i == 1 {
                curve.velocity = 0.;
            }
        }

        for curve in self.curves.iter_mut() {
            if ctx.keyboard.is_key_pressed(curve.move_keys.right) {
                curve.rotate_right()
            }

            if ctx.keyboard.is_key_pressed(curve.move_keys.left) {
                curve.rotate_left()
            }

            let now = std::time::Instant::now();

            // Disable trail if countdown is done and invulnerability countdown
            if now.duration_since(curve.trail_ts) > curve.trail_countdown {
                curve.trail_active = false;
                curve.trail_ts = now;
            }

            // Enable trail if countdown is done
            if now.duration_since(curve.trail_ts) > INV_DURATION && !curve.trail_active {
                curve.trail_active = true;
                curve.trail_countdown = new_trail_countdown();
                curve.trail_ts = now;
            }

            if curve.trail_active {
                // Push the line to the actual curve
                let line = Line::interpolate(curve.position, curve.next_pos());
                curve.lines.push(line);
            }

            curve.mv();
        }
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
