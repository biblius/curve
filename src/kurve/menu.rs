use super::curve::{Curve, MoveKeys};
use super::PAUSE_MENU_CENTER;
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

pub trait PlayerConfigMod {
    fn apply(&self, kurve: &mut Kurve, ctx: &mut Context) -> GameResult;

    fn update(&mut self, ctx: &mut Context);

    fn draw(&self, ctx: &mut Context, canvas: &mut Canvas, paused: bool);
}

pub struct KurveMenu {
    /// Available menu items
    pub items: Vec<KurveMenuItem>,

    /// The selected item
    pub selected: usize,

    /// Available default colors
    pub colors: Vec<Color>,

    /// Available default movement keys
    pub keys: Vec<MoveKeys>,

    /// The active player config modifier
    pub active_mod: Option<Box<dyn PlayerConfigMod>>,
}

impl KurveMenu {
    /// Handle selected elements subcommand
    pub fn navigate(&mut self, ctx: &mut Context) {
        if ctx.keyboard.is_key_just_pressed(KeyCode::Right) {
            // Moves all config selectors cause it's more intuitive
            for item in self.items.iter_mut() {
                if let KurveMenuItem::PlayerCurveConfig(conf) = item {
                    conf.selected = conf.selected.next();
                }
            }
        }

        if ctx.keyboard.is_key_just_pressed(KeyCode::Left) {
            for item in self.items.iter_mut() {
                if let KurveMenuItem::PlayerCurveConfig(conf) = item {
                    conf.selected = conf.selected.previous();
                }
            }
        }
    }
    /// Select the current item and activate its modifier. The caller must ensure that the selected
    /// item is a player config.
    pub fn select_item(&self) -> Option<SelectAction> {
        let KurveMenuItem::PlayerCurveConfig(ref conf) = self.items[self.selected] else {
            unreachable!("called in wrong context")
        };

        match conf.selected {
            PlayerConfigFocus::Name => Some(SelectAction::Modifier(Box::new(PlayerNameModifier {
                buf: String::new(),
            }))),
            PlayerConfigFocus::Color => {
                if !self.colors.is_empty() {
                    Some(SelectAction::Modifier(Box::new(PlayerColorModifier::new(
                        self.colors.clone(),
                    ))))
                } else {
                    None
                }
            }
            PlayerConfigFocus::Keys => {
                Some(SelectAction::Modifier(Box::new(PlayerKeyModifier::new())))
            }
            PlayerConfigFocus::Remove => Some(SelectAction::RemovePlayer),
        }
    }

    pub fn decrement_config_ids(&mut self, from: usize) {
        for config in self.items[from..].iter_mut() {
            if let KurveMenuItem::PlayerCurveConfig(cfg) = config {
                cfg.id = cfg.id.saturating_sub(1);
            }
        }
    }

    pub fn draw_setup(&self, ctx: &mut Context, canvas: &mut Canvas, paused: bool) -> GameResult {
        let (x, y) = ctx.gfx.drawable_size();

        let center = if paused {
            Self::center_pause((x, y))
        } else {
            Self::center_setup((x, y))
        };

        for (i, item) in self.items.iter().enumerate() {
            let selected = self.selected == i;

            match item {
                KurveMenuItem::PlayerCurveConfig(config) => {
                    self.draw_player_cfg(ctx, canvas, config, center, selected, i as f32)?;
                }
                KurveMenuItem::AddPlayer => {
                    self.draw_add_player(
                        ctx,
                        canvas,
                        center,
                        selected,
                        i as f32,
                        self.colors.is_empty(),
                    )?;
                }
                KurveMenuItem::Start => {
                    if !paused {
                        self.draw_start_game(ctx, canvas, center, selected)?;
                    }
                }
            }
        }

        if let Some(ref modif) = self.active_mod {
            modif.draw(ctx, canvas, paused)
        }

        Ok(())
    }

    #[inline]
    fn center_setup((x, y): (f32, f32)) -> Point2<f32> {
        Point2 {
            x: x * SETUP_MENU_CENTER.0,
            y: y * SETUP_MENU_CENTER.1,
        }
    }

    #[inline]
    fn center_pause((x, y): (f32, f32)) -> Point2<f32> {
        Point2 {
            x: x * PAUSE_MENU_CENTER.0,
            y: y * PAUSE_MENU_CENTER.1,
        }
    }

    fn draw_player_cfg(
        &self,
        ctx: &mut Context,
        canvas: &mut Canvas,
        config: &PlayerConfig,
        center: Point2<f32>,
        selected: bool,
        offset: f32,
    ) -> GameResult {
        const NAME_OFFSET: f32 = 0.03;
        const KEYS_OFFSET: f32 = 0.35;
        const COLOR_OFFSET: f32 = 0.5;
        const REMOVE_OFFSET: f32 = 0.8;
        let PlayerConfig {
            name,
            color,
            keys,
            selected: sub_selected,
            ..
        } = config;
        let (x, y) = ctx.gfx.drawable_size();

        let size = (x * 0.4, y * 0.05);

        // Full rect for item

        let rect = graphics::Rect::new(
            center.x - size.0 * 0.5,
            y * 0.3 + offset * 75.,
            size.0,
            size.1,
        );

        // Player name

        let mut name = graphics::Text::new(name);
        name.set_scale(PxScale::from(24.));
        let mut name_rect = name.dimensions(ctx).unwrap();
        canvas.draw(
            &name,
            DrawParam::default().dest(Point2 {
                x: rect.x + size.0 * NAME_OFFSET,
                y: rect.y + size.1 * 0.5 - name_rect.h * 0.5,
            }),
        );

        // Player keys

        let mut keys = graphics::Text::new(keys.to_string());
        keys.set_scale(PxScale::from(24.));
        let mut keys_rect = keys.dimensions(ctx).unwrap();
        canvas.draw(
            &keys,
            DrawParam::default().dest(Point2 {
                x: rect.x + size.0 * KEYS_OFFSET - keys_rect.w * 0.5,
                y: rect.y + size.1 * 0.5 - keys_rect.h * 0.5,
            }),
        );

        // Player color

        let mut color_rect = graphics::Rect::new(
            rect.x + rect.w * COLOR_OFFSET,
            rect.y + rect.h * 0.25,
            rect.h * 0.5,
            rect.h * 0.5,
        );

        let color_mesh =
            graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), color_rect, *color)?;

        canvas.draw(&color_mesh, DrawParam::default());

        // Remove player

        let mut remove_rect = graphics::Rect::new(
            rect.x + rect.w * REMOVE_OFFSET,
            rect.y + rect.h * 0.5,
            rect.h * 0.5,
            rect.h * 0.1,
        );

        let remove_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            remove_rect,
            Color::WHITE,
        )?;

        canvas.draw(&remove_mesh, DrawParam::default());

        // If currently selected draw the select boxes

        if selected {
            let border_mesh =
                graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::stroke(2.), rect, *color)?;

            canvas.draw(&border_mesh, DrawParam::default());

            match sub_selected {
                PlayerConfigFocus::Name => {
                    let adjust = (name_rect.w * 1.1 - name_rect.w) * 0.5;
                    name_rect.w *= 1.1;
                    name_rect.h *= 1.2;
                    let inner_border_mesh = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::stroke(2.),
                        name_rect,
                        *color,
                    )?;
                    canvas.draw(
                        &inner_border_mesh,
                        DrawParam::default().dest(Point2 {
                            x: rect.x + size.0 * NAME_OFFSET - adjust,
                            y: rect.y + size.1 * 0.5 - name_rect.h * 0.5,
                        }),
                    );
                }
                PlayerConfigFocus::Color => {
                    let adjust_x = (color_rect.w * 1.2 - color_rect.w) * 0.5;
                    let adjust_y = (color_rect.h * 1.2 - color_rect.h) * 0.5;
                    color_rect.w *= 1.2;
                    color_rect.x -= adjust_x;
                    color_rect.h *= 1.2;
                    color_rect.y -= adjust_y;
                    let inner_border_mesh = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::stroke(2.),
                        color_rect,
                        *color,
                    )?;
                    canvas.draw(
                        &inner_border_mesh,
                        DrawParam::default(), // Rect pos is already set
                    );
                }
                PlayerConfigFocus::Keys => {
                    keys_rect.w *= 1.1;
                    keys_rect.h *= 1.2;
                    let inner_border_mesh = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::stroke(2.),
                        keys_rect,
                        *color,
                    )?;
                    canvas.draw(
                        &inner_border_mesh,
                        DrawParam::default().dest(Point2 {
                            x: rect.x + size.0 * KEYS_OFFSET - keys_rect.w * 0.5,
                            y: rect.y + size.1 * 0.5 - keys_rect.h * 0.5,
                        }),
                    );
                }
                PlayerConfigFocus::Remove => {
                    let adjust_x = (remove_rect.w * 1.4 - remove_rect.w) * 0.5;
                    let adjust_y = (remove_rect.h * 3. - remove_rect.h) * 0.5;
                    remove_rect.w *= 1.4;
                    remove_rect.x -= adjust_x;
                    remove_rect.h *= 3.;
                    remove_rect.y -= adjust_y;
                    let inner_border_mesh = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::stroke(2.),
                        remove_rect,
                        *color,
                    )?;
                    canvas.draw(
                        &inner_border_mesh,
                        DrawParam::default(), // Rect pos is already set
                    );
                }
            }
        }

        Ok(())
    }

    fn draw_add_player(
        &self,
        ctx: &mut Context,
        canvas: &mut Canvas,
        center: Point2<f32>,
        selected: bool,
        offset: f32,
        disabled: bool,
    ) -> GameResult {
        let (x, y) = ctx.gfx.drawable_size();

        let size = (x * 0.05, y * 0.03);

        let rect = graphics::Rect::new(
            center.x - size.0 * 0.5,
            y * 0.3 + offset * 75.,
            size.0,
            size.1,
        );

        let mut text = graphics::Text::new("+");
        text.set_scale(PxScale::from(24.));
        text.fragments_mut().iter_mut().for_each(|frag| {
            frag.color = Some(if disabled {
                Color {
                    r: 0.5,
                    g: 0.5,
                    b: 0.5,
                    a: 0.8,
                }
            } else {
                Color::WHITE
            })
        });
        let text_dims = text.dimensions(ctx).unwrap();

        canvas.draw(
            &text,
            DrawParam::default().dest(Point2 {
                x: rect.x + size.0 * 0.5 - text_dims.w * 0.5,
                y: rect.y + size.1 * 0.5 - text_dims.h * 0.5,
            }),
        );

        if selected {
            let mesh = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::stroke(2.),
                rect,
                if disabled {
                    Color {
                        r: 0.5,
                        g: 0.5,
                        b: 0.5,
                        a: 0.8,
                    }
                } else {
                    Color::WHITE
                },
            )?;

            canvas.draw(&mesh, DrawParam::default());
        }
        Ok(())
    }

    fn draw_start_game(
        &self,
        ctx: &mut Context,
        canvas: &mut Canvas,
        center: Point2<f32>,
        selected: bool,
    ) -> GameResult {
        let (x, y) = ctx.gfx.drawable_size();

        let size = (x * 0.1, y * 0.03);

        let rect = graphics::Rect::new(
            center.x - size.0 * 0.5,
            y - size.1 * 0.5 - y * 0.25,
            size.0,
            size.1,
        );

        let mut text = graphics::Text::new("Start");
        text.set_scale(PxScale::from(24.));
        let text_dims = text.dimensions(ctx).unwrap();

        canvas.draw(
            &text,
            DrawParam::default().dest(Point2 {
                x: rect.x + size.0 * 0.5 - text_dims.w * 0.5,
                y: rect.y + size.1 * 0.5 - text_dims.h * 0.5,
            }),
        );

        if selected {
            let mesh = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::stroke(2.),
                rect,
                Color::WHITE,
            )?;

            canvas.draw(&mesh, DrawParam::default());
        }

        Ok(())
    }
}

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

pub enum SelectAction {
    Modifier(Box<dyn PlayerConfigMod>),
    RemovePlayer,
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
        curve.move_keys = *keys;
        curve.color = *color;
        curve.mesh = Curve::create_mesh(ctx, *color)?;
        Ok(())
    }

    /// Create a player curve pair from the config. Bounds are necessary for the spawned curve.
    pub fn to_player_curve_pair(
        &self,
        ctx: &mut Context,
        bounds: ArenaBounds,
        alive: bool,
        velocity: f32,
    ) -> Result<(Player, Curve), GameError> {
        let player = Player::new(self.name.clone(), self.keys);

        let mesh = Curve::create_mesh(ctx, self.color)?;

        let curve = Curve::new_random_pos(
            self.id,
            bounds,
            player.move_keys,
            mesh,
            self.color,
            alive,
            velocity,
        );

        Ok((player, curve))
    }
}

#[derive(Debug)]
pub enum PlayerConfigFocus {
    Name,
    Color,
    Keys,
    Remove,
}

impl PlayerConfigFocus {
    pub fn next(&self) -> Self {
        match self {
            Self::Name => Self::Keys,
            Self::Keys => Self::Color,
            Self::Color => Self::Remove,
            Self::Remove => Self::Name,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Self::Name => Self::Remove,
            Self::Keys => Self::Name,
            Self::Color => Self::Keys,
            Self::Remove => Self::Color,
        }
    }
}

#[derive(Debug)]
pub struct PlayerNameModifier {
    /// Current text buffer
    pub buf: String,
}

impl PlayerConfigMod for PlayerNameModifier {
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

    fn draw(&self, ctx: &mut Context, canvas: &mut Canvas, paused: bool) {
        let (x, y) = ctx.gfx.drawable_size();

        let center = if paused {
            modifier_center_pause(x, y)
        } else {
            modifier_center_setup(x, y)
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

impl PlayerConfigMod for PlayerKeyModifier {
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

    fn draw(&self, ctx: &mut Context, canvas: &mut Canvas, paused: bool) {
        let (x, y) = ctx.gfx.drawable_size();

        let center = if paused {
            modifier_center_pause(x, y)
        } else {
            modifier_center_setup(x, y)
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

impl PlayerConfigMod for PlayerColorModifier {
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

    fn draw(&self, ctx: &mut Context, canvas: &mut Canvas, paused: bool) {
        let (x, y) = ctx.gfx.drawable_size();

        let center = if paused {
            modifier_center_pause(x, y)
        } else {
            modifier_center_setup(x, y)
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

fn modifier_center_setup(x: f32, y: f32) -> Point2<f32> {
    Point2 {
        x: x * SETUP_MENU_CENTER.0,
        y: y * SETUP_MENU_CENTER.1 - y * 0.035,
    }
}

fn modifier_center_pause(x: f32, y: f32) -> Point2<f32> {
    Point2 {
        x: x * PAUSE_MENU_CENTER.0,
        y: y * PAUSE_MENU_CENTER.1 - y * 0.035,
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RotationDirection {
    Cw,
    Ccw,
}
