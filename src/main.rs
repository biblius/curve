use std::f32::consts::{FRAC_PI_2, FRAC_PI_8, PI};

use ggez::event::{self};
use ggez::graphics::{DrawParam, InstanceArray};
use ggez::input::keyboard::KeyCode;
use ggez::mint::Point2;
use ggez::{graphics, graphics::Color};
use ggez::{Context, GameResult};
use rand::Rng;

const GRID_SIZE: f32 = 3.;
const CURVE_SIZE: f32 = GRID_SIZE * 0.5;

const ROT_SPEED: f32 = FRAC_PI_8 * 0.1;
const VELOCITY: f32 = 3.;

struct GameArena {
    size_x: f32,
    size_y: f32,
    center: Point2<f32>,
}

impl GameArena {
    fn new_default(size_x: f32, size_y: f32) -> Self {
        let center = Point2 {
            x: size_x * 0.5,
            y: size_y * 0.5,
        };

        GameArena {
            size_x,
            size_y,
            center,
        }
    }
}

#[derive(Debug)]
struct MoveKeys {
    left: KeyCode,
    right: KeyCode,
}

#[derive(Debug)]
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
        let rot: f32 = rng.gen_range(0f32..2. * PI);
        Self {
            position: Point2 { x: p_x, y: p_y },
            rotation: rot,
            velocity: VELOCITY,
            move_keys: mv_keys,
        }
    }

    fn new(pos: Point2<f32>, rot: f32, mv_keys: MoveKeys) -> Self {
        Self {
            position: pos,
            rotation: rot,
            velocity: VELOCITY,
            move_keys: mv_keys,
        }
    }

    fn rotate(&mut self, dir: RotDirection) {
        match dir {
            RotDirection::Right => self.rotation += ROT_SPEED,
            RotDirection::Left => self.rotation -= ROT_SPEED,
        }
    }

    fn mv(&mut self) {
        self.position.x += self.velocity * self.rotation.cos();
        self.position.y += self.velocity * self.rotation.sin();
    }

    fn next_pos(&self) -> Point2<f32> {
        Point2 {
            x: self.position.x + self.velocity * self.rotation.cos(),
            y: self.position.y + self.velocity * self.rotation.sin(),
        }
    }
}

enum RotDirection {
    Left,
    Right,
}

/// A line obtained from interpolation
struct Line(Vec<Point2<f32>>);

struct GameState {
    game_arena: GameArena,
    curves: Vec<Curve>,

    /// Used to draw the curves, a one dimensional repr of `lines`
    trails: Vec<InstanceArray>,

    /// The curves for game logic
    lines: Vec<Line>,

    players: usize,
}

impl GameState {
    pub fn new(ctx: &mut Context) -> Self {
        let (screen_w, screen_h) = ctx.gfx.drawable_size();

        //Arena
        let game_arena = GameArena::new_default(screen_w, screen_h);

        let player1 = Curve::new(
            Point2 {
                x: game_arena.center.x + 100.,
                y: game_arena.center.y,
            },
            0.,
            MoveKeys {
                left: KeyCode::Q,
                right: KeyCode::W,
            },
        );

        let player2 = Curve::new(
            Point2 {
                x: game_arena.center.x - 100.,
                y: game_arena.center.y,
            },
            FRAC_PI_2,
            MoveKeys {
                left: KeyCode::Left,
                right: KeyCode::Down,
            },
        );

        Self {
            curves: vec![player1, player2],
            game_arena,
            trails: vec![InstanceArray::new(ctx, None), InstanceArray::new(ctx, None)],
            lines: vec![],
            players: 2,
        }
    }
}

fn bounding_box(p: Point2<f32>) -> Vec<Point2<f32>> {
    let r = Point2 {
        x: (p.x + 1.).round(),
        y: p.y.round(),
    };
    let tr = Point2 {
        x: (p.x + 1.).round(),
        y: (p.y - 1.).round(),
    };
    let t = Point2 {
        x: p.x.round(),
        y: (p.y - 1.).round(),
    };
    let tl = Point2 {
        x: (p.x - 1.).round(),
        y: (p.y - 1.).round(),
    };
    let l = Point2 {
        x: (p.x - 1.).round(),
        y: p.y.round(),
    };
    let bl = Point2 {
        x: (p.x - 1.).round(),
        y: (p.y + 1.).round(),
    };
    let b = Point2 {
        x: p.x.round(),
        y: (p.y + 1.).round(),
    };
    let br = Point2 {
        x: (p.x + 1.).round(),
        y: (p.y + 1.).round(),
    };

    vec![p, r, tr, t, tl, l, bl, b, br]
}

fn interpolate_points(origin: Point2<f32>, target: Point2<f32>) -> Vec<Point2<f32>> {
    let mut points = vec![];
    let d_x = target.x - origin.x;
    let d_y = target.y - origin.y;
    let max = d_x.abs().max(d_y.abs()).max(1.);

    let step_x = d_x / max;
    let step_y = d_y / max;
    let mut i = 0.;
    while i < max {
        let pos_x = origin.x + i * step_x;
        let pos_y = origin.y + i * step_y;
        points.push(Point2 {
            x: pos_x.round(),
            y: pos_y.round(),
        });
        i += 1.;
    }

    points
}

impl event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let mut new_lines = 0;
        for i in 0..self.players {
            // Append the trail
            let curve = &mut self.curves[i];
            let trail = &mut self.trails[i];

            if ctx.keyboard.is_key_pressed(curve.move_keys.right) {
                curve.rotate(RotDirection::Right)
            }

            if ctx.keyboard.is_key_pressed(curve.move_keys.left) {
                curve.rotate(RotDirection::Left)
            }

            let ps = interpolate_points(curve.position, curve.next_pos());
            for p in ps.iter() {
                trail.push((*p).into());
            }

            self.lines.push(Line(ps));
            new_lines += 1;

            curve.mv();
        }

        for Line(pts) in self.lines[0..self.lines.len() - new_lines].iter() {
            for curve in self.curves.iter_mut() {
                let bbox = bounding_box(curve.next_pos());
                for bp in bbox.iter() {
                    for pt in pts {
                        if pt.x == bp.x && pt.y == bp.y {
                            curve.velocity = 0.;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, None);

        let arena_rect = graphics::Rect::new(
            -self.game_arena.size_x * 0.5,
            -self.game_arena.size_y * 0.5,
            self.game_arena.size_x,
            self.game_arena.size_y,
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

        let draw_param = graphics::DrawParam::default().dest(self.game_arena.center);
        canvas.draw(&arena_mesh, draw_param);

        for (i, curve) in self.curves.iter().enumerate() {
            let trail = &self.trails[i];
            canvas.draw_instanced_mesh(curve_mesh.clone(), trail, DrawParam::default());

            draw_param.dest(curve.position);
            canvas.draw(&curve_mesh, draw_param);

            let c_rect =
                graphics::Rect::new(-CURVE_SIZE, -CURVE_SIZE, CURVE_SIZE * 2., CURVE_SIZE * 2.);
            let c_mesh =
                graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), c_rect, Color::RED)?;

            let bbox = bounding_box(curve.next_pos());
            for bbox in bbox {
                canvas.draw(&c_mesh, draw_param.dest(bbox));
            }
        }

        canvas.finish(ctx)?;

        Ok(())
    }
}

fn main() -> GameResult {
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
