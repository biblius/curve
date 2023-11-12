use curve::MoveKeys;
use ggez::conf::WindowMode;
use ggez::event::{self};
use ggez::graphics::{self, Canvas, Color, DrawParam, Drawable, PxScale};
use ggez::input::keyboard::KeyCode;
use ggez::mint::Point2;
use ggez::{Context, GameError, GameResult};
use kurve::Kurve;
use rand::Rng;
use std::f32::consts::FRAC_PI_8;
use std::fmt::Debug;
use std::time::Duration;

mod curve;
mod kurve;
mod point;

const CURVE_SIZE: f32 = 2.;

const ROT_SPEED: f32 = FRAC_PI_8 * 0.1;
const VELOCITY: f32 = 1.;

const TRAIL_INTERVAL_MIN: u64 = 2000;
const TRAIL_INTERVAL_MAX: u64 = 4000;

/// 2-3 players
const SIZE_SMALL: (f32, f32) = (0.35, 0.55);

/// 4-6 players
const SIZE_MED: (f32, f32) = (0.5, 0.5);

const WINNER_GLOAT_DURATION: Duration = Duration::from_secs(3);

/// Curve invulnerability duration when it is not leaving the trail
const INV_DURATION: Duration = Duration::from_millis(300);

/// Get a random duration for counting down the segment skip in the curves
pub fn new_trail_countdown() -> Duration {
    let mut rng = rand::thread_rng();
    let millis = rng.gen_range(TRAIL_INTERVAL_MIN..TRAIL_INTERVAL_MAX);
    Duration::from_millis(millis)
}

#[derive(Debug, Default)]
pub struct Player {
    score: u8,
    name: String,
    move_keys: MoveKeys,
}

impl Player {
    pub fn new(name: String, move_keys: MoveKeys) -> Self {
        Self {
            score: 0,
            name,
            move_keys,
        }
    }
}

#[derive(Debug)]
pub struct MainMenu {
    items: [MainMenuItem; 1],
    selected: usize,
}

impl MainMenu {
    pub fn new() -> Self {
        Self {
            items: [MainMenuItem::PlayButton { size: (200., 60.) }],
            selected: 0,
        }
    }

    fn draw(&self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
        let (x, y) = ctx.gfx.drawable_size();
        let center = Point2 {
            x: x * 0.5,
            y: y * 0.5,
        };

        for item in self.items.iter() {
            match item {
                MainMenuItem::PlayButton { size } => {
                    let rect = graphics::Rect::new(
                        center.x - size.0 * 0.5,
                        center.y - size.1 * 0.5,
                        size.0,
                        size.1,
                    );

                    let mut text = graphics::Text::new("Play");
                    text.set_scale(PxScale::from(24.));
                    let text_dims = text.dimensions(ctx).unwrap();

                    canvas.draw(
                        &text,
                        DrawParam::default().dest(Point2 {
                            x: rect.x + size.0 * 0.5 - text_dims.w * 0.5,
                            y: rect.y + size.1 * 0.5 - text_dims.h * 0.5,
                        }),
                    );

                    let mesh = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::stroke(2.),
                        rect,
                        Color::WHITE,
                    )?;

                    canvas.draw(&mesh, DrawParam::default());
                }
            }
        }

        Ok(())
    }
}

impl Default for MainMenu {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum MainMenuItem {
    PlayButton { size: (f32, f32) },
}

#[derive(Debug)]
enum GameState {
    MainMenu,
    Kurve,
}

pub struct Game {
    main_menu: MainMenu,

    kurve: Kurve,

    state: GameState,
}

impl Game {
    pub fn new(ctx: &mut Context) -> Result<Self, GameError> {
        let kurve = Kurve::new(ctx)?;

        Ok(Self {
            main_menu: MainMenu::new(),
            kurve,
            state: GameState::MainMenu,
        })
    }
}

impl event::EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        match self.state {
            GameState::MainMenu => {
                if !ctx.keyboard.is_key_pressed(KeyCode::Return) {
                    return Ok(());
                }

                match self.main_menu.items[self.main_menu.selected] {
                    MainMenuItem::PlayButton { .. } => self.state = GameState::Kurve,
                }
            }
            GameState::Kurve => {
                self.kurve.update(ctx)?;
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Some(Color::BLACK));

        match self.state {
            GameState::MainMenu => self.main_menu.draw(ctx, &mut canvas)?,
            GameState::Kurve => self.kurve.draw(ctx, &mut canvas)?,
        }

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

    let state = Game::new(&mut ctx)?;
    event::run(ctx, event_loop, state);
}

#[macro_export]
macro_rules! key_to_str {
    ($ctx:ident, $focus:ident) => {
        $crate::key_to_str!($ctx, $focus,
            KeyCode::Key1 => '1',
            KeyCode::Key2 => '2',
            KeyCode::Key3 => '3',
            KeyCode::Key4 => '4',
            KeyCode::Key5 => '5',
            KeyCode::Key6 => '6',
            KeyCode::Key7 => '7',
            KeyCode::Key8 => '8',
            KeyCode::Key9 => '9',
            KeyCode::Key0 => '0',
            KeyCode::A => 'a',
            KeyCode::B => 'b',
            KeyCode::C => 'c',
            KeyCode::D => 'd',
            KeyCode::E => 'e',
            KeyCode::F => 'f',
            KeyCode::G => 'g',
            KeyCode::H => 'h',
            KeyCode::I => 'i',
            KeyCode::J => 'j',
            KeyCode::K => 'k',
            KeyCode::L => 'l',
            KeyCode::M => 'm',
            KeyCode::N => 'n',
            KeyCode::O => 'o',
            KeyCode::P => 'p',
            KeyCode::Q => 'q',
            KeyCode::R => 'r',
            KeyCode::S => 's',
            KeyCode::T => 't',
            KeyCode::U => 'u',
            KeyCode::V => 'v',
            KeyCode::W => 'w',
            KeyCode::X => 'x',
            KeyCode::Y => 'y',
            KeyCode::Z => 'z',
            KeyCode::Space => ' '
        )
    };

    ($ctx:ident, $focus:ident, $($id:path => $ch:literal),*) => {
        $(
            if $ctx.keyboard.is_key_just_pressed($id) {
                if $ctx.keyboard.is_mod_active(KeyMods::SHIFT) && $ch.is_ascii_alphabetic() {
                    $focus.buf.push($ch.to_ascii_uppercase());
                }

                $focus.buf.push($ch);
            }
        )*
    };
}
/*
fn key_to_char(key: KeyCode) -> Option<char> {
    match key {
        KeyCode::Key1 => Some('1'),
        KeyCode::Key2 => Some('2'),
        KeyCode::Key3 => Some('3'),
        KeyCode::Key4 => Some('4'),
        KeyCode::Key5 => Some('5'),
        KeyCode::Key6 => Some('6'),
        KeyCode::Key7 => Some('7'),
        KeyCode::Key8 => Some('8'),
        KeyCode::Key9 => Some('9'),
        KeyCode::Key0 => Some('0'),
        KeyCode::A => Some('a'),
        KeyCode::B => Some('b'),
        KeyCode::C => Some('c'),
        KeyCode::D => Some('d'),
        KeyCode::E => Some('e'),
        KeyCode::F => Some('f'),
        KeyCode::G => Some('g'),
        KeyCode::H => Some('h'),
        KeyCode::I => Some('i'),
        KeyCode::J => Some('j'),
        KeyCode::K => Some('k'),
        KeyCode::L => Some('l'),
        KeyCode::M => Some('m'),
        KeyCode::N => Some('n'),
        KeyCode::O => Some('o'),
        KeyCode::P => Some('p'),
        KeyCode::Q => Some('q'),
        KeyCode::R => Some('r'),
        KeyCode::S => Some('s'),
        KeyCode::T => Some('t'),
        KeyCode::U => Some('u'),
        KeyCode::V => Some('v'),
        KeyCode::W => Some('w'),
        KeyCode::X => Some('x'),
        KeyCode::Y => Some('y'),
        KeyCode::Z => Some('z'),
        KeyCode::Space => Some(' '),
        _ => None,
    }
}
 */
