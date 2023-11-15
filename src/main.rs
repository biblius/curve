use game::Game;
use ggez::conf::WindowMode;
use ggez::event::{self};
use ggez::GameResult;
use std::f32::consts::FRAC_PI_8;
use std::time::Duration;

mod game;
mod kurve;
mod menu;

const CURVE_SIZE: f32 = 2.;

const ROT_SPEED: f32 = FRAC_PI_8 * 0.1;
const VELOCITY: f32 = 60.;

const TRAIL_INTERVAL_MIN: u64 = 2000;
const TRAIL_INTERVAL_MAX: u64 = 4000;

/// 2-3 players
const SIZE_SMALL: (f32, f32) = (0.35, 0.55);

/// 4-6 players
// const SIZE_MED: (f32, f32) = (0.5, 0.5);

const WINNER_GLOAT_DURATION: Duration = Duration::from_secs(3);

/// Curve invulnerability duration when it is not leaving the trail
const INV_DURATION: Duration = Duration::from_millis(300);

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
                if $ctx.keyboard.is_mod_active(ggez::input::keyboard::KeyMods::SHIFT) && $ch.is_ascii_alphabetic() {
                    $focus.buf.push($ch.to_ascii_uppercase());
                } else {
                    $focus.buf.push($ch);
                }
            }
        )*
    };
}

use ggez::input::keyboard::KeyCode;

pub fn display_key(key: KeyCode) -> Option<&'static str> {
    match key {
        KeyCode::Key1 => Some("1"),
        KeyCode::Key2 => Some("2"),
        KeyCode::Key3 => Some("3"),
        KeyCode::Key4 => Some("4"),
        KeyCode::Key5 => Some("5"),
        KeyCode::Key6 => Some("6"),
        KeyCode::Key7 => Some("7"),
        KeyCode::Key8 => Some("8"),
        KeyCode::Key9 => Some("9"),
        KeyCode::Key0 => Some("0"),
        KeyCode::A => Some("a"),
        KeyCode::B => Some("b"),
        KeyCode::C => Some("c"),
        KeyCode::D => Some("d"),
        KeyCode::E => Some("e"),
        KeyCode::F => Some("f"),
        KeyCode::G => Some("g"),
        KeyCode::H => Some("h"),
        KeyCode::I => Some("i"),
        KeyCode::J => Some("j"),
        KeyCode::K => Some("k"),
        KeyCode::L => Some("l"),
        KeyCode::M => Some("m"),
        KeyCode::N => Some("n"),
        KeyCode::O => Some("o"),
        KeyCode::P => Some("p"),
        KeyCode::Q => Some("q"),
        KeyCode::R => Some("r"),
        KeyCode::S => Some("s"),
        KeyCode::T => Some("t"),
        KeyCode::U => Some("u"),
        KeyCode::V => Some("v"),
        KeyCode::W => Some("w"),
        KeyCode::X => Some("x"),
        KeyCode::Y => Some("y"),
        KeyCode::Z => Some("z"),
        KeyCode::Space => Some(" "),
        KeyCode::Escape => Some("ESC"),
        KeyCode::F1 => Some("F1"),
        KeyCode::F2 => Some("F2"),
        KeyCode::F3 => Some("F3"),
        KeyCode::F4 => Some("F4"),
        KeyCode::F5 => Some("F5"),
        KeyCode::F6 => Some("F6"),
        KeyCode::F7 => Some("F7"),
        KeyCode::F8 => Some("F8"),
        KeyCode::F9 => Some("F9"),
        KeyCode::F10 => Some("F10"),
        KeyCode::F11 => Some("F11"),
        KeyCode::F12 => Some("F12"),
        KeyCode::F13 => Some("F13"),
        KeyCode::F14 => Some("F14"),
        KeyCode::F15 => Some("F15"),
        KeyCode::F16 => Some("F16"),
        KeyCode::F17 => Some("F17"),
        KeyCode::F18 => Some("F18"),
        KeyCode::F19 => Some("F19"),
        KeyCode::F20 => Some("F20"),
        KeyCode::F21 => Some("F21"),
        KeyCode::F22 => Some("F22"),
        KeyCode::F23 => Some("F23"),
        KeyCode::F24 => Some("F24"),
        KeyCode::Snapshot => Some("PrtSrc"),
        KeyCode::Scroll => Some("ScrLock"),
        KeyCode::Pause => Some("Pause"),
        KeyCode::Insert => Some("Insert"),
        KeyCode::Home => Some("Home"),
        KeyCode::Delete => Some("Del"),
        KeyCode::End => Some("End"),
        KeyCode::PageDown => Some("PgDn"),
        KeyCode::PageUp => Some("PgUp"),
        KeyCode::Left => Some("Left"),
        KeyCode::Up => Some("Up"),
        KeyCode::Right => Some("Right"),
        KeyCode::Down => Some("Down"),
        KeyCode::Back => Some("Backspace"),
        KeyCode::Numlock => Some(""),
        KeyCode::Numpad0 => Some("Num0"),
        KeyCode::Numpad1 => Some("Num1"),
        KeyCode::Numpad2 => Some("Num2"),
        KeyCode::Numpad3 => Some("Num3"),
        KeyCode::Numpad4 => Some("Num4"),
        KeyCode::Numpad5 => Some("Num5"),
        KeyCode::Numpad6 => Some("Num6"),
        KeyCode::Numpad7 => Some("Num7"),
        KeyCode::Numpad8 => Some("Num8"),
        KeyCode::Numpad9 => Some("Num9"),
        KeyCode::NumpadAdd => Some("+"),
        KeyCode::NumpadDivide => Some("/"),
        KeyCode::NumpadDecimal => Some("."),
        KeyCode::NumpadComma => Some(","),
        KeyCode::NumpadEnter => Some("NumEnter"),
        KeyCode::NumpadEquals => Some("="),
        KeyCode::NumpadMultiply => Some("*"),
        KeyCode::NumpadSubtract => Some("-"),
        KeyCode::Apostrophe => Some("'"),
        KeyCode::Asterisk => Some("*"),
        KeyCode::At => Some("@"),
        KeyCode::Backslash => Some("\\"),
        KeyCode::Colon => Some(":"),
        KeyCode::Comma => Some(","),
        KeyCode::Equals => Some("="),
        _ => None,
    }
}
