use super::curve::{Curve, MoveKeys};
use super::{player::Player, ArenaBounds, Kurve, SETUP_MENU_CENTER};
use crate::{display_key, key_to_str};
use ggez::GameResult;
use ggez::{
    graphics::{self, Canvas, Color, DrawParam, Drawable, PxScale},
    input::keyboard::KeyCode,
    mint::Point2,
    Context, GameError,
};
use std::fmt::Debug;

pub trait ModifierElement {
    fn apply(&self, kurve: &mut Kurve, ctx: &mut Context) -> GameResult;

    fn update(&mut self, ctx: &mut Context);

    fn draw(&self, ctx: &mut Context, canvas: &mut Canvas);
}

pub struct KurveMenu {
    pub items: Vec<KurveMenuItem>,
    pub selected: usize,
    pub colors: Vec<Color>,
    pub keys: Vec<MoveKeys>,
    pub active_mod: Option<Box<dyn ModifierElement>>,
}

impl KurveMenu {}

impl Debug for KurveMenu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KurveMenu")
            .field("items", &self.items)
            .field("selected", &self.selected)
            .field("colors", &self.colors)
            .field("keys", &self.keys)
            .finish()
    }
}

#[derive(Debug)]
pub enum KurveMenuItem {
    PlayerCurveConfig(PlayerConfig),
    AddPlayer,
    Start,
}

#[derive(Debug)]
pub struct PlayerConfig {
    /// The index into the players and curves vec
    pub id: usize,
    pub name: String,
    pub color: Color,
    pub keys: MoveKeys,
    pub selected: PlayerConfigFocus,
}

impl PlayerConfig {
    pub fn apply(&self, ctx: &mut Context, player: &mut Player, curve: &mut Curve) -> GameResult {
        let Self {
            name, color, keys, ..
        } = self;

        player.name = name.clone();
        player.move_keys = *keys;
        curve.color = *color;
        curve.mesh = Curve::create_mesh(ctx, *color)?;
        Ok(())
    }

    /// Create a player curve pair from the config. Bounds are necessary for the spawned curve.
    pub fn to_player_curve_pair(
        &self,
        ctx: &mut Context,
        bounds: ArenaBounds,
    ) -> Result<(Player, Curve), GameError> {
        let player = Player::new(self.name.clone(), self.keys);

        let mesh = Curve::create_mesh(ctx, self.color)?;

        let curve = Curve::new_random_pos(self.id, bounds, player.move_keys, mesh, self.color);

        Ok((player, curve))
    }
}

#[derive(Debug)]
pub enum PlayerConfigFocus {
    Name,
    Color,
    Keys,
}

impl PlayerConfigFocus {
    pub fn next(&self) -> Self {
        match self {
            Self::Name => Self::Keys,
            Self::Keys => Self::Color,
            Self::Color => Self::Name,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Self::Name => Self::Color,
            Self::Keys => Self::Name,
            Self::Color => Self::Keys,
        }
    }
}

#[derive(Debug)]
pub struct PlayerNameModifier {
    /// Current text buffer
    pub buf: String,
}

impl ModifierElement for PlayerNameModifier {
    fn apply(&self, kurve: &mut Kurve, ctx: &mut Context) -> GameResult {
        let (config, player, curve) = kurve.extract_cfg_player_curve();
        config.name = self.buf.clone();
        config.apply(ctx, player, curve)?;
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) {
        if ctx.keyboard.is_key_pressed(KeyCode::Back) {
            self.buf.pop();
            return;
        }

        if self.buf.len() <= 20 {
            key_to_str!(ctx, self);
        }
    }

    fn draw(&self, ctx: &mut Context, canvas: &mut Canvas) {
        let (x, y) = ctx.gfx.drawable_size();
        let center = Point2 {
            x: x * SETUP_MENU_CENTER.0,
            y: y * SETUP_MENU_CENTER.1 + 650.,
        };

        let size = (300., 50.);

        let rect = graphics::Rect::new(
            center.x - size.0 * 0.5,
            center.y - size.1 * 0.5,
            size.0,
            size.1,
        );

        let mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            rect,
            Color::from_rgb(30, 30, 30),
        )
        .unwrap();

        let mut name = graphics::Text::new(&self.buf);
        name.set_scale(PxScale::from(24.));

        let mut banner = graphics::Text::new("Enter name");
        banner.set_scale(PxScale::from(18.));

        let text_dims = name.dimensions(ctx).unwrap();
        let banner_dims = banner.dimensions(ctx).unwrap();

        canvas.draw(
            &banner,
            DrawParam::default().dest(Point2 {
                x: rect.x,
                y: rect.y - banner_dims.h,
            }),
        );

        canvas.draw(&mesh, DrawParam::default());

        canvas.draw(
            &name,
            DrawParam::default().dest(Point2 {
                x: rect.x + size.0 * 0.5 - text_dims.w * 0.5,
                y: rect.y + size.1 * 0.5 - text_dims.h * 0.5,
            }),
        );
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PlayerKeyModifier {
    dir: RotationDirection,
    key_ccw: KeyCode,
    key_cw: KeyCode,
}

impl PlayerKeyModifier {
    pub fn new() -> Self {
        Self {
            dir: RotationDirection::Ccw,
            key_ccw: KeyCode::Asterisk,
            key_cw: KeyCode::Asterisk,
        }
    }
}

impl ModifierElement for PlayerKeyModifier {
    fn apply(&self, kurve: &mut Kurve, ctx: &mut Context) -> GameResult {
        let (config, player, curve) = kurve.extract_cfg_player_curve();
        config.keys = (*self).into();
        config.apply(ctx, player, curve)?;
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) {
        if ctx.keyboard.is_key_just_pressed(KeyCode::Back) {
            match self.dir {
                RotationDirection::Ccw => {}
                RotationDirection::Cw => {
                    self.dir = RotationDirection::Ccw;
                    self.key_cw = KeyCode::Asterisk;
                }
            }
            return;
        }

        if let Some(key) = ctx.keyboard.pressed_keys().iter().next() {
            if ctx.keyboard.is_key_just_pressed(*key) {
                match self.dir {
                    RotationDirection::Cw => self.key_cw = *key,
                    RotationDirection::Ccw => self.key_ccw = *key,
                }
                self.dir = RotationDirection::Cw;
            }
        }
    }

    fn draw(&self, ctx: &mut Context, canvas: &mut Canvas) {
        let (x, y) = ctx.gfx.drawable_size();
        let center = Point2 {
            x: x * SETUP_MENU_CENTER.0,
            y: y * SETUP_MENU_CENTER.1 + 650.,
        };

        let size = (50., 50.);

        // Left key

        let rect1 = graphics::Rect::new(
            center.x - size.0 * 1.5,
            center.y - size.1 * 0.5,
            size.0,
            size.1,
        );

        let mesh1 = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            rect1,
            Color::from_rgb(30, 30, 30),
        )
        .unwrap();

        // Right key

        let rect2 = graphics::Rect::new(
            center.x + size.0 * 0.5,
            center.y - size.1 * 0.5,
            size.0,
            size.1,
        );

        let mesh2 = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            rect2,
            Color::from_rgb(30, 30, 30),
        )
        .unwrap();

        // The descriptions

        let mut ccw_banner = graphics::Text::new("CCW");
        ccw_banner.set_scale(PxScale::from(18.));
        let ccw_banner_dims = ccw_banner.dimensions(ctx).unwrap();

        let mut cw_banner = graphics::Text::new("CW");
        cw_banner.set_scale(PxScale::from(18.));
        let cw_banner_dims = cw_banner.dimensions(ctx).unwrap();

        // The input keys

        let mut key_cw = graphics::Text::new(display_key(self.key_cw).unwrap_or("???"));
        key_cw.set_scale(PxScale::from(24.));
        let cw_dims = key_cw.dimensions(ctx).unwrap();

        let mut key_ccw = graphics::Text::new(display_key(self.key_ccw).unwrap_or("???"));
        key_ccw.set_scale(PxScale::from(24.));
        let ccw_dims = key_ccw.dimensions(ctx).unwrap();

        canvas.draw(
            &ccw_banner,
            DrawParam::default().dest(Point2 {
                x: rect1.x + rect1.w * 0.5 - ccw_banner_dims.w * 0.5,
                y: rect1.y - ccw_banner_dims.h,
            }),
        );

        canvas.draw(
            &cw_banner,
            DrawParam::default().dest(Point2 {
                x: rect2.x + rect2.w * 0.5 - cw_banner_dims.w * 0.5,
                y: rect2.y - cw_banner_dims.h,
            }),
        );

        canvas.draw(&mesh1, DrawParam::default());
        canvas.draw(&mesh2, DrawParam::default());

        let mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::stroke(1.),
            match self.dir {
                RotationDirection::Cw => rect2,
                RotationDirection::Ccw => rect1,
            },
            Color::from_rgb(30, 30, 30),
        )
        .unwrap();

        canvas.draw(&mesh, DrawParam::default());

        canvas.draw(
            &key_ccw,
            DrawParam::default().dest(Point2 {
                x: rect1.x + rect1.w * 0.5 - ccw_dims.w * 0.5,
                y: rect1.y + rect1.h * 0.5 - ccw_dims.h * 0.5,
            }),
        );

        canvas.draw(
            &key_cw,
            DrawParam::default().dest(Point2 {
                x: rect2.x + rect2.w * 0.5 - cw_dims.w * 0.5,
                y: rect2.y + rect2.h * 0.5 - cw_dims.h * 0.5,
            }),
        );
    }
}

impl From<PlayerKeyModifier> for MoveKeys {
    fn from(value: PlayerKeyModifier) -> Self {
        Self {
            cw: value.key_cw,
            ccw: value.key_ccw,
        }
    }
}

#[derive(Debug)]
pub struct PlayerColorModifier {
    colors: Vec<Color>,
    selected: usize,
}

impl PlayerColorModifier {
    pub fn new(colors: Vec<Color>) -> Self {
        Self {
            colors,
            selected: 0,
        }
    }
}

impl ModifierElement for PlayerColorModifier {
    fn apply(&self, kurve: &mut Kurve, ctx: &mut Context) -> GameResult {
        let (config, player, curve) = kurve.extract_cfg_player_curve();
        let current = curve.color;

        config.color = self.colors[self.selected];
        config.apply(ctx, player, curve)?;

        if let Some(idx) = kurve.menu.colors.iter().position(|c| *c == current) {
            kurve.menu.colors.remove(idx);
            kurve.menu.colors.insert(idx, current);
        }

        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) {
        if ctx.keyboard.is_key_just_pressed(KeyCode::Left) {
            if self.selected == 0 {
                self.selected = self.colors.len() - 1;
            } else {
                self.selected -= 1;
            }
        }
        if ctx.keyboard.is_key_just_pressed(KeyCode::Right) {
            self.selected = (self.selected + 1) % self.colors.len();
        }
    }

    fn draw(&self, ctx: &mut Context, canvas: &mut Canvas) {
        let (x, y) = ctx.gfx.drawable_size();
        let center = Point2 {
            x: x * SETUP_MENU_CENTER.0,
            y: y * SETUP_MENU_CENTER.1 + 650.,
        };

        let size = (50., 50.);

        let rect = graphics::Rect::new(
            center.x - size.0 * 0.5,
            center.y - size.1 * 0.5,
            size.0,
            size.1,
        );

        let mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            rect,
            self.colors[self.selected],
        )
        .unwrap();

        canvas.draw(&mesh, DrawParam::default());
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RotationDirection {
    Cw,
    Ccw,
}
