use std::f32::consts::{FRAC_PI_2, FRAC_PI_8, PI};

use ggez::event::{self};
use ggez::graphics::{DrawParam, InstanceArray};
use ggez::input::keyboard::KeyCode;
use ggez::mint::Point2;
use ggez::{graphics, graphics::Color};
use ggez::{Context, GameResult};
use rand::Rng;

const GRID_SIZE: f32 = 3.;
const CURVE_SIZE: f32 = GRID_SIZE / 2.;
const ROTATION_MAX: f32 = PI * 2.;
const ROT_SPEED: f32 = FRAC_PI_8 / 8.;
const VELOCITY: f32 = 1.;

fn clamp(value: &mut f32, low: f32, high: f32) {
    if *value < low {
        *value = low
    }
    if *value > high {
        *value = high
    }
}

struct GameArena {
    size_x: f32,
    size_y: f32,
    x_half: f32,
    y_half: f32,
    center: Point2<f32>,
    top: f32,
    left: f32,
    bottom: f32,
    right: f32,
}

impl GameArena {
    fn new_default(size_x: f32, size_y: f32, ctx: &mut Context) -> Self {
        let (screen_x, screen_y) = ctx.gfx.drawable_size();
        let center = Point2 {
            x: screen_x * 0.5,
            y: screen_y * 0.5,
        };
        GameArena {
            size_x,
            size_y,
            x_half: size_x * 0.5,
            y_half: size_y * 0.5,
            center,
            top: center.y - size_y * 0.5,
            left: center.x - size_x * 0.5,
            bottom: center.y + size_y * 0.5,
            right: center.x + size_x * 0.5,
        }
    }
}

struct MoveKeys {
    left: KeyCode,
    right: KeyCode,
}

struct Curve {
    /// Where the curve is located
    position: Point2<f32>,

    /// Rotation angle in rad
    rotation: f32,

    /// How fast the curve is moving
    velocity: f32,

    move_keys: MoveKeys,
}

impl Curve {
    fn new_random_pos(x_min: f32, x_max: f32, y_min: f32, y_max: f32, mv_keys: MoveKeys) -> Self {
        let mut rng = rand::thread_rng();
        let p_x: f32 = rng.gen_range(x_min..x_max);
        let p_y: f32 = rng.gen_range(y_min..y_max);
        let rot: f32 = rng.gen_range(0f32..ROTATION_MAX);
        Self {
            position: Point2 { x: p_x, y: p_y },
            rotation: rot,
            velocity: VELOCITY,
            move_keys: mv_keys,
        }
    }

    fn new(x: f32, y: f32, rot: f32, mv_keys: MoveKeys) -> Self {
        Self {
            position: Point2 { x, y },
            rotation: rot,
            velocity: VELOCITY,
            move_keys: mv_keys,
        }
    }

    fn rotate(&mut self, dir: RotDirection) {
        match dir {
            RotDirection::Right => self.rotation = (self.rotation + ROT_SPEED) % ROTATION_MAX,
            RotDirection::Left => self.rotation = (self.rotation - ROT_SPEED) % ROTATION_MAX,
        }
    }

    fn mv(&mut self) {
        self.position.x += self.velocity * self.rotation.cos();
        self.position.y += self.velocity * self.rotation.sin();
    }

    /*     fn next_pos(&self) -> Point2<f32> {
        Point2 {
            x: self.position.x + self.velocity * self.rotation.cos(),
            y: self.position.y + self.velocity * self.rotation.sin(),
        }
    } */
}

enum RotDirection {
    Left,
    Right,
}

struct GameState {
    game_arena: GameArena,
    curves: Vec<Curve>,
    trails: Vec<InstanceArray>,
    context_center: Point2<f32>,
}

impl GameState {
    pub fn new(ctx: &mut Context) -> Self {
        let (screen_w, screen_h) = ctx.gfx.drawable_size();

        let (screen_w_half, screen_h_half) = (screen_w * 0.5, screen_h * 0.5);

        let context_center = Point2 {
            x: screen_w_half,
            y: screen_h_half,
        };

        //Arena
        let game_arena = GameArena::new_default(screen_w, screen_h, ctx);

        Self {
            curves: vec![
                Curve::new(
                    game_arena.x_half - 100.,
                    game_arena.y_half,
                    0.,
                    MoveKeys {
                        left: KeyCode::Q,
                        right: KeyCode::W,
                    },
                ),
                Curve::new(
                    game_arena.x_half + 100.,
                    game_arena.y_half,
                    FRAC_PI_2,
                    MoveKeys {
                        left: KeyCode::Left,
                        right: KeyCode::Down,
                    },
                ),
            ],
            game_arena,
            context_center,
            trails: vec![InstanceArray::new(ctx, None), InstanceArray::new(ctx, None)],
        }
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        for (i, curve) in self.curves.iter_mut().enumerate() {
            // Append the trail
            let trail = &mut self.trails[i];
            trail.push(curve.position.into());

            if ctx.keyboard.is_key_pressed(curve.move_keys.right) {
                curve.rotate(RotDirection::Right)
            }

            if ctx.keyboard.is_key_pressed(curve.move_keys.left) {
                curve.rotate(RotDirection::Left)
            }

            curve.mv();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, None);

        let arena_rect = graphics::Rect::new(
            -self.game_arena.x_half,
            -self.game_arena.y_half,
            self.game_arena.size_x,
            self.game_arena.size_y,
        );

        let arena_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            arena_rect,
            Color::BLACK,
        )?;

        let curve_mesh = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Point2 { x: 0., y: 0. },
            CURVE_SIZE,
            0.1,
            Color::WHITE,
        )?;

        let draw_param = graphics::DrawParam::default().dest(self.game_arena.center);
        canvas.draw(&arena_mesh, draw_param);

        for (i, curve) in self.curves.iter().enumerate() {
            let trail = &self.trails[i];
            /*
            draw_param.dest(curve.position);
            canvas.draw(&curve_mesh, draw_param); */

            canvas.draw_instanced_mesh(curve_mesh.clone(), trail, DrawParam::default());
        }

        let mut x = 0.;
        while x < self.game_arena.size_x {
            let line = graphics::Mesh::new_line(
                ctx,
                &[
                    Point2 { x, y: 0. },
                    Point2 {
                        x,
                        y: self.game_arena.size_y,
                    },
                ],
                1.,
                Color::RED,
            )?;
            canvas.draw(&line, DrawParam::default());
            x += GRID_SIZE;
        }

        canvas.finish(ctx)?;

        Ok(())
    }
}

fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("curve", "biblius");
    let (mut ctx, event_loop) = cb.build()?;
    ctx.gfx.set_window_title("curve");

    let state = GameState::new(&mut ctx);
    event::run(ctx, event_loop, state);
}
