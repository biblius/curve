use crate::kurve::Kurve;
use crate::menu::{MainMenu, MainMenuItem};
use ggez::audio::{SoundSource, Source};
use ggez::event::{self};
use ggez::graphics::{self, Color, Image};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::{Context, GameError, GameResult};
use std::fmt::Debug;

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
        /*         Source::new(ctx, "/httm.mp3")
        .unwrap()
        .play_detached(ctx)
        .unwrap(); */
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
                if !ctx.keyboard.is_key_just_pressed(KeyCode::Return) {
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

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: KeyInput,
        _repeated: bool,
    ) -> Result<(), GameError> {
        if input.keycode == Some(KeyCode::Escape) && matches!(self.state, GameState::MainMenu) {
            ctx.request_quit();
        }
        Ok(())
    }
}
