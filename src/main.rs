use curve::{Curve, MoveKeys};
use ggez::conf::WindowMode;
use ggez::event::{self};
use ggez::graphics::{self, Color, DrawParam, Drawable};
use ggez::input::keyboard::KeyCode;
use ggez::mint::Point2;
use ggez::{Context, GameResult};
use kurve::{ArenaBounds, Kurve};
use rand::Rng;
use std::f32::consts::FRAC_PI_8;
use std::fmt::{Debug, Write};
use std::time::Duration;

mod curve;
mod kurve;
mod point;

const GRID_SIZE: f32 = 3.;
const CURVE_SIZE: f32 = GRID_SIZE * 0.5;

const ROT_SPEED: f32 = FRAC_PI_8 * 0.1;
const VELOCITY: f32 = 1.;

const ARENA_W_MOD: f32 = 0.3;
const ARENA_H_MOD: f32 = 0.5;

const TRAIL_INTERVAL_MIN: u64 = 2000;
const TRAIL_INTERVAL_MAX: u64 = 4000;

const WINNER_GLOAT_DURATION: Duration = Duration::from_secs(3);

/// Curve invulnerability duration when it is not leaving the trail
const INV_DURATION: Duration = Duration::from_millis(300);

/// Get a random duration for counting down the segment skip in the curves
pub fn new_trail_countdown() -> Duration {
    let mut rng = rand::thread_rng();
    let millis = rng.gen_range(TRAIL_INTERVAL_MIN..TRAIL_INTERVAL_MAX);
    Duration::from_millis(millis)
}

#[derive(Debug)]
pub struct Player {
    score: u8,
    name: String,
}

#[derive(Debug)]
pub struct MainMenu {}

#[derive(Debug)]
pub struct CreateGameMenu {
    items: Vec<MenuItem>,
}

pub struct MenuItem {
    prev: Option<Box<Self>>,
    next: Option<Box<Self>>,

    on_select: Box<dyn FnMut(Game)>,
}

impl Debug for MenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuItem")
            .field("prev", &self.prev)
            .field("next", &self.next)
            .field("on_select", &"{ .. }")
            .finish()
    }
}

#[derive(Debug)]
struct Game {
    //    current_menu: MainMenu,
    kurve: Kurve,

    players: Vec<Player>,
}

impl Game {
    pub fn new(ctx: &mut Context) -> Self {
        let (size_x, size_y) = ctx.gfx.drawable_size();

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

        let player1 = Player {
            name: "Bedga".to_string(),
            score: 0,
        };
        let player2 = Player {
            name: "Mitz".to_string(),
            score: 0,
        };

        let players = vec![player1, player2];

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
        let kurve = Kurve::new(size, vec![player1, player2], bounds);

        Self { kurve, players }
    }
}

impl event::EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if let Some(winner) = self.kurve.update(ctx) {
            self.players[winner].score += 1;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Some(Color::BLACK));
        let (sizex, sizey) = ctx.gfx.drawable_size();

        self.kurve.draw(ctx, &mut canvas)?;

        let mut score_text = String::new();

        for player in self.players.iter() {
            writeln!(score_text, "{}: {}", player.name, player.score).unwrap();
        }

        let score_text = graphics::Text::new(score_text);
        let score_rect = score_text.dimensions(ctx).unwrap();

        let draw_param = DrawParam::default().dest(Point2 {
            x: sizex * 0.5 - score_rect.w * 0.5,
            y: 30.0,
        });

        canvas.draw(&score_text, draw_param);

        canvas.finish(ctx)?;

        Ok(())
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("curve", "biblius");
    let (mut ctx, event_loop) = cb.build()?;
    ctx.gfx.set_window_title("curve");
    let res = ctx
        .gfx
        .supported_resolutions()
        .next()
        .unwrap()
        .to_logical(1.);
    ctx.gfx
        .set_mode(
            WindowMode::default()
                .dimensions(res.width, res.height)
                .fullscreen_type(ggez::conf::FullscreenType::Desktop),
        )
        .unwrap();
    ctx.gfx.set_drawable_size(res.width, res.height).unwrap();

    let state = Game::new(&mut ctx);
    event::run(ctx, event_loop, state);
}
