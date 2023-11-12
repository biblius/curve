use std::f32::consts::{FRAC_PI_8, PI};
use std::time::{Duration, Instant};

use crate::curve::MoveKeys;
use crate::point::Line;
use crate::{curve::Curve, point::BoundingBox, CURVE_SIZE};
use crate::{key_to_str, new_trail_countdown, Player, SIZE_SMALL, VELOCITY, WINNER_GLOAT_DURATION};
use ggez::graphics::{Drawable, PxScale};
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::GameError;
use ggez::{
    graphics::{self, Canvas, Color, DrawParam, InstanceArray},
    mint::Point2,
    Context, GameResult,
};
use rand::distributions::uniform::SampleUniform;
use rand::Rng;
use std::fmt::{Debug, Write};

const COLORS: [Color; 5] = [
    Color::GREEN,
    Color::YELLOW,
    Color::MAGENTA,
    Color::CYAN,
    Color {
        r: 1.,
        g: 0.1,
        b: 0.1,
        a: 1.,
    },
];

const MOVE_KEYS: [MoveKeys; 5] = [
    MoveKeys {
        left: KeyCode::PageUp,
        right: KeyCode::PageDown,
    },
    MoveKeys {
        left: KeyCode::J,
        right: KeyCode::K,
    },
    MoveKeys {
        left: KeyCode::V,
        right: KeyCode::B,
    },
    MoveKeys {
        left: KeyCode::O,
        right: KeyCode::P,
    },
    MoveKeys {
        left: KeyCode::Q,
        right: KeyCode::W,
    },
];

/// Multipliers for the x and y axis used to position the kurve area during setup
const SETUP_KURVE_CENTER: (f32, f32) = (0.7, 0.5);

/// Multipliers for the x and y axis used to position the menu during setup
const SETUP_MENU_CENTER: (f32, f32) = (0.3, 0.3);

/// To normalise the menu
const SETUP_MENU_TICK: Duration = Duration::from_millis(60);

/// Represents the current phase of the game
#[derive(Debug)]
pub enum KurveState {
    /// The game is currently being prepared
    Setup,

    /// The game is prepared and waiting to launch
    StartCountdown {
        /// When this phase has started
        started: Instant,
    },

    /// The game is running
    Running,

    /// The game is paused
    Paused,

    /// The game is gloating the winner
    Winner {
        /// When this phase has started
        started: Instant,

        /// The player index
        id: usize,
    },
}

pub struct KurveMenu {
    items: Vec<KurveMenuItem>,
    selected: usize,
    colors: Vec<Color>,
    keys: Vec<MoveKeys>,
    active_mod: Option<Box<dyn ModifierElement>>,
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

#[derive(Debug)]
pub struct FocusBuffer {
    item_id: usize,
    /// Current text buffer
    buf: String,
}

impl ModifierElement for FocusBuffer {
    fn apply(&self, kurve: &mut Kurve) {
        let item = &mut kurve.menu.items[kurve.menu.selected];
        let KurveMenuItem::PlayerCurveConfig(config) = item else {
            panic!("string modifier being applied to unsupported item");
        };
        config.name = self.buf.clone();
        kurve.players[config.id].name = config.name.clone();
    }

    fn update(&mut self, ctx: &mut Context) {
        if ctx.keyboard.is_key_pressed(KeyCode::Back) {
            self.buf.pop();
            return;
        }

        key_to_str!(ctx, self);
    }

    fn target(&self) -> usize {
        self.item_id
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
    id: usize,
    name: String,
    color: Color,
    keys: MoveKeys,
    selected: PlayerConfigFocus,
}

#[derive(Debug)]
enum PlayerConfigFocus {
    Name,
    Color,
    Keys,
}

impl PlayerConfig {
    /// Create a player curve pair from the config. Bounds are necessary for the spawned curve.
    fn to_player_curve_pair(
        &self,
        ctx: &mut Context,
        bounds: ArenaBounds,
    ) -> Result<(Player, Curve), GameError> {
        let player = Player::new(self.name.clone(), self.keys);

        let mesh = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Point2 { x: 0., y: 0. },
            CURVE_SIZE,
            0.1,
            self.color,
        )?;

        let curve = Curve::new_random_pos(self.id, bounds, player.move_keys, mesh, self.color);

        Ok((player, curve))
    }
}

trait ModifierElement {
    fn apply(&self, kurve: &mut Kurve);

    fn update(&mut self, ctx: &mut Context);

    fn target(&self) -> usize;
}

/// Achtung die main game struct.
#[derive(Debug)]
pub struct Kurve {
    /// Where the arena starts and ends on each axis
    pub bounds: ArenaBounds,

    /// Players involved in the game
    pub players: Vec<Player>,

    /// The curves in the game. It is very important the indices
    /// here match the players.
    pub curves: Vec<Curve>,

    /// Current game state
    pub state: KurveState,

    pub menu: KurveMenu,
}

/// Game logic implementations
impl Kurve {
    pub fn new(ctx: &mut Context) -> Result<Self, GameError> {
        let size = ctx.gfx.drawable_size();

        let mut colors = COLORS.to_vec();
        let mut keys = MOVE_KEYS.to_vec();

        let config1 = PlayerConfig {
            id: 0,
            name: "Player 1".to_string(),
            color: colors.pop().unwrap(),
            keys: keys.pop().unwrap(),
            selected: PlayerConfigFocus::Name,
        };

        let config2 = PlayerConfig {
            id: 1,
            name: "Player 2".to_string(),
            color: colors.pop().unwrap(),
            keys: keys.pop().unwrap(),
            selected: PlayerConfigFocus::Name,
        };

        let bounds = ArenaBounds::new(
            Point2 {
                x: size.0 * SETUP_KURVE_CENTER.0,
                y: size.1 * SETUP_KURVE_CENTER.1,
            },
            size,
            SIZE_SMALL,
        );

        let (player1, curve1) = config1.to_player_curve_pair(ctx, bounds)?;
        let (player2, curve2) = config2.to_player_curve_pair(ctx, bounds)?;

        Ok(Self {
            bounds,
            curves: vec![curve1, curve2],
            players: vec![player1, player2],
            state: KurveState::Setup,
            menu: KurveMenu {
                items: vec![
                    KurveMenuItem::PlayerCurveConfig(config1),
                    KurveMenuItem::PlayerCurveConfig(config2),
                    KurveMenuItem::AddPlayer,
                    KurveMenuItem::Start,
                ],
                selected: 0,
                colors,
                keys,
                active_mod: None,
            },
        })
    }

    /// Add a player to the game and return their index
    fn add_player(&mut self, player: Player, curve: Curve) {
        self.players.push(player);
        self.curves.push(curve);
    }

    fn set_bounds(&mut self, bounds: ArenaBounds) {
        self.bounds = bounds;
    }

    #[inline]
    fn toggle_pause(&mut self) {
        match self.state {
            KurveState::Running => self.state = KurveState::Paused,
            KurveState::Paused => self.state = KurveState::Running,
            _ => {}
        }
    }

    fn tick_winner(&mut self, ctx: &mut Context, started: Instant) {
        let now = Instant::now();

        // Process movement
        for curve in self.curves.iter_mut() {
            curve.rotate(ctx);
            curve.tick_trail();
            curve.mv();
        }

        if now.duration_since(started) >= WINNER_GLOAT_DURATION {
            self.reset_curves();
            self.state = KurveState::StartCountdown {
                started: Instant::now(),
            };
        }
    }

    /// Tick the round countdown
    fn tick_countdown(&mut self, ctx: &mut Context, started: Instant) {
        for curve in self.curves.iter_mut() {
            curve.rotate(ctx);
        }
        let now = Instant::now();
        if now.duration_since(started) >= WINNER_GLOAT_DURATION {
            for curve in self.curves.iter_mut() {
                curve.trail_ts = Instant::now();
            }
            self.state = KurveState::Running;
        }
    }

    /// Reset the curves' positions and liveness
    fn reset_curves(&mut self) {
        for curve in self.curves.iter_mut() {
            curve.position = self.bounds.random_pos();
            curve.alive = true;
            curve.rotation = random_rot();
            curve.lines.clear();
            curve.trail_active = true;
            curve.trail_countdown = new_trail_countdown();
            curve.velocity = VELOCITY;
        }
    }

    /// Process the setup menu
    fn tick_setup_menu(&mut self, ctx: &mut Context) -> GameResult {
        if self.menu.active_mod.is_some() && ctx.keyboard.is_key_just_pressed(KeyCode::Escape) {
            self.menu.active_mod = None;
        }

        if self.menu.active_mod.is_some() && ctx.keyboard.is_key_just_pressed(KeyCode::Return) {
            let focus = self.menu.active_mod.take().unwrap();
            focus.apply(self);
            return Ok(());
        }

        if let Some(ref mut focus) = self.menu.active_mod {
            focus.update(ctx);

            return Ok(());
        }

        // Handle Enter

        if ctx.keyboard.is_key_just_pressed(KeyCode::Return) {
            let item = &self.menu.items[self.menu.selected];
            match item {
                KurveMenuItem::PlayerCurveConfig(conf) => match conf.selected {
                    PlayerConfigFocus::Name => {
                        self.menu.active_mod = Some(Box::new(FocusBuffer {
                            item_id: self.menu.selected,
                            buf: String::new(),
                        }))
                    }
                    PlayerConfigFocus::Color => todo!(),
                    PlayerConfigFocus::Keys => todo!(),
                },
                KurveMenuItem::AddPlayer => {
                    let id = self.players.len();

                    let config = PlayerConfig {
                        id,
                        name: format!("Player {}", id + 1),
                        color: self.menu.colors.pop().unwrap(),
                        keys: self.menu.keys.pop().unwrap(),
                        selected: PlayerConfigFocus::Name,
                    };
                    let (player, curve) = config.to_player_curve_pair(ctx, self.bounds)?;

                    self.add_player(player, curve);

                    let mut idx = 0;
                    let mut items = self.menu.items.iter();

                    while matches!(items.next(), Some(KurveMenuItem::PlayerCurveConfig(_))) {
                        idx += 1;
                    }

                    self.menu
                        .items
                        .insert(idx, KurveMenuItem::PlayerCurveConfig(config))
                }
                KurveMenuItem::Start => {
                    let size = ctx.gfx.drawable_size();
                    self.set_bounds(ArenaBounds::new_center(size, SIZE_SMALL));
                    self.reset_curves();
                    self.state = KurveState::StartCountdown {
                        started: Instant::now(),
                    }
                }
            }
        }

        if ctx.keyboard.is_key_just_pressed(KeyCode::Up) {
            if self.menu.selected == 0 {
                self.menu.selected = self.menu.items.len() - 1;
            } else {
                self.menu.selected -= 1;
            }
        }

        if ctx.keyboard.is_key_just_pressed(KeyCode::Down) {
            self.menu.selected = (self.menu.selected + 1) % self.menu.items.len()
        }

        Ok(())
    }

    /// Process the setup stagin area
    fn tick_setup_curves(&mut self, ctx: &mut Context) {
        // Calculate wall collisions
        for curve in self.curves.iter_mut() {
            let bbox = BoundingBox::new(curve.next_pos());
            if let Some(collision) =
                check_border_axis_collision(self.bounds.x_min, self.bounds.x_max, bbox.xs())
            {
                match collision {
                    Collision::Min => {
                        curve.position.x = self.bounds.x_max;
                    }
                    Collision::Max => {
                        curve.position.x = self.bounds.x_min;
                    }
                }
            }

            if let Some(collision) =
                check_border_axis_collision(self.bounds.y_min, self.bounds.y_max, bbox.ys())
            {
                match collision {
                    Collision::Min => {
                        curve.position.y = self.bounds.y_max;
                    }
                    Collision::Max => {
                        curve.position.y = self.bounds.y_min;
                    }
                }
            }

            curve.rotate(ctx);

            curve.tick_trail();

            curve.mv();

            if curve.lines.len() > 20 {
                curve.lines.pop_front();
            }
        }
    }

    /// Process a running game's tick
    fn tick_running(&mut self, ctx: &mut Context) -> Option<usize> {
        // Bitflags for collision
        let mut collisions = 0u8;

        // Calculate collisions
        for (i, curve) in self.curves.iter().enumerate() {
            if !curve.alive {
                continue;
            }

            let bbox = BoundingBox::new(curve.next_pos());

            if check_border_collision(
                self.bounds.x_min,
                self.bounds.x_max,
                self.bounds.y_min,
                self.bounds.y_max,
                bbox,
            ) {
                collisions |= 1 << i;
                continue;
            }

            for (j, curve) in self.curves.iter().enumerate() {
                let lines = &curve.lines;

                // Skip the last line of the current curve due to self collision
                let line_count = if i == j {
                    lines.len().saturating_sub(1)
                } else {
                    lines.len()
                };

                for (_, line) in lines
                    .iter()
                    .enumerate()
                    .take_while(|(i, _)| *i < line_count)
                {
                    if check_line_collision(bbox, line) {
                        collisions |= 1 << i;
                    }
                }
            }
        }

        // Apply collisions
        for (i, curve) in self.curves.iter_mut().enumerate() {
            if collisions >> i == 1 {
                curve.velocity = 0.;
                curve.alive = false;
            }
        }

        // Check for winners
        if let Some(winner) = self.check_winner() {
            return Some(winner);
        }

        // Process movement
        for curve in self.curves.iter_mut() {
            curve.rotate(ctx);

            curve.tick_trail();

            curve.mv();
        }

        None
    }

    /// Check whether there is only one curve currently alive
    fn check_winner(&self) -> Option<usize> {
        let mut winner = None;
        let mut alive = 0;

        for curve in self.curves.iter() {
            if curve.alive {
                if alive < 1 {
                    winner = Some(curve.player_id);
                }
                alive += 1;
            }

            if alive > 1 {
                winner = None;
                break;
            }
        }

        winner
    }

    /// Update the game state
    pub fn update(&mut self, ctx: &mut Context) -> GameResult {
        if ctx.keyboard.is_key_pressed(KeyCode::Space) {
            self.toggle_pause();
        }

        match self.state {
            KurveState::Setup => {
                self.tick_setup_menu(ctx)?;
                self.tick_setup_curves(ctx);
            }
            KurveState::Running => {
                if let Some(winner) = self.tick_running(ctx) {
                    self.state = KurveState::Winner {
                        started: Instant::now(),
                        id: winner,
                    };
                    self.players[winner].score += 1;
                }
            }
            KurveState::StartCountdown { started } => self.tick_countdown(ctx, started),
            KurveState::Winner { started, .. } => self.tick_winner(ctx, started),
            KurveState::Paused => {}
        }

        Ok(())
    }
}

/// Drawing logic impls
impl Kurve {
    pub fn draw(&self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
        let arena_rect = graphics::Rect::new(
            self.bounds.x_min,
            self.bounds.y_min,
            self.bounds.x_max - self.bounds.x_min,
            self.bounds.y_max - self.bounds.y_min,
        );

        let arena_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            arena_rect,
            Color::from_rgb(30, 30, 30),
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

            canvas.draw_instanced_mesh(curve.mesh.clone(), &trail, draw_param);

            canvas.draw(&curve.mesh, draw_param.dest(curve.position));

            /*             let c_rect =
                graphics::Rect::new(-CURVE_SIZE, -CURVE_SIZE, CURVE_SIZE * 2., CURVE_SIZE * 2.);
            let c_mesh =
                graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), c_rect, Color::RED)?;

            let bbox = BoundingBox::new(curve.next_pos());
            for bbox in bbox {
                canvas.draw(&c_mesh, draw_param.dest(bbox));
            } */
        }

        if let KurveState::Setup = self.state {
            self.draw_setup_menu(ctx, canvas)?;
        }

        if let KurveState::StartCountdown { started } = self.state {
            self.draw_countdown_phase(ctx, canvas, started)?;
        }

        if let KurveState::Winner { id, .. } = self.state {
            self.draw_winner_phase(ctx, canvas, &self.players[id].name)
        }

        self.draw_score(ctx, canvas);

        Ok(())
    }

    pub fn draw_setup_menu(&self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
        let (x, y) = ctx.gfx.drawable_size();
        let center = Point2 {
            x: x * SETUP_MENU_CENTER.0,
            y: y * SETUP_MENU_CENTER.1,
        };

        for (i, item) in self.menu.items.iter().enumerate() {
            let selected = self.menu.selected == i;

            match item {
                KurveMenuItem::PlayerCurveConfig(PlayerConfig {
                    name,
                    color,
                    selected: sub_selected,
                    ..
                }) => {
                    let size = (600., 100.);

                    let rect = graphics::Rect::new(
                        center.x - size.0 * 0.5,
                        center.y - size.1 * 0.5 + i as f32 * 100.,
                        size.0,
                        size.1,
                    );

                    let mut text = graphics::Text::new(name);

                    text.set_scale(PxScale::from(24.));
                    let text_dims = text.dimensions(ctx).unwrap();

                    canvas.draw(
                        &text,
                        DrawParam::default().dest(Point2 {
                            x: rect.x + size.0 * 0.1,
                            y: rect.y + size.1 * 0.5 - text_dims.h * 0.5,
                        }),
                    );

                    if selected {
                        let mesh = graphics::Mesh::new_rectangle(
                            ctx,
                            graphics::DrawMode::stroke(2.),
                            rect,
                            *color,
                        )?;

                        canvas.draw(&mesh, DrawParam::default());
                    }
                }
                KurveMenuItem::AddPlayer => {
                    let size = (50., 50.);

                    let rect = graphics::Rect::new(
                        center.x - size.0 * 0.5,
                        center.y - size.1 * 0.5 + i as f32 * 100.,
                        size.0,
                        size.1,
                    );

                    let mut text = graphics::Text::new("+");
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
                }
                KurveMenuItem::Start => {
                    let size = (200., 50.);

                    let rect = graphics::Rect::new(
                        center.x - size.0 * 0.5,
                        center.y - size.1 * 0.5 + i as f32 * 100.,
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
                }
            }
        }

        Ok(())
    }

    /// Display the counter in the middle of the screen on countdown
    fn draw_countdown_phase(
        &self,
        ctx: &mut Context,
        canvas: &mut Canvas,
        started: Instant,
    ) -> GameResult {
        let (x, y) = ctx.gfx.drawable_size();

        // Draw the countdown
        let second = (WINNER_GLOAT_DURATION.saturating_sub(Instant::now().duration_since(started)))
            .as_secs()
            + 1;

        let mut text = graphics::Text::new(second.to_string());
        text.set_scale(PxScale::from(24.));

        let rect = text.dimensions(ctx).unwrap();

        canvas.draw(
            &text,
            DrawParam::default().dest(Point2 {
                x: x * 0.5 - rect.w * 0.5,
                y: y * 0.5,
            }),
        );

        // Draw the lines displaying rotations
        for curve in self.curves.iter() {
            let pos_point = curve.position;
            let rot_point = curve.project_rotation();
            let line =
                graphics::Mesh::new_line(ctx, &[pos_point, rot_point], 1., curve.color).unwrap();
            let tip = graphics::Mesh::new_polygon(
                ctx,
                graphics::DrawMode::fill(),
                &[
                    Point2 {
                        x: rot_point.x + 7. * (curve.rotation + PI - FRAC_PI_8 * 0.6).cos(),
                        y: rot_point.y + 7. * (curve.rotation + PI - FRAC_PI_8 * 0.6).sin(),
                    },
                    Point2 {
                        x: rot_point.x,
                        y: rot_point.y,
                    },
                    Point2 {
                        x: rot_point.x + 7. * (curve.rotation + PI + FRAC_PI_8 * 0.6).cos(),
                        y: rot_point.y + 7. * (curve.rotation + PI + FRAC_PI_8 * 0.6).sin(),
                    },
                ],
                curve.color,
            )?;
            canvas.draw(&line, DrawParam::default());
            canvas.draw(&tip, DrawParam::default());
        }
        Ok(())
    }

    fn draw_winner_phase(&self, ctx: &mut Context, canvas: &mut Canvas, player_name: &str) {
        let (x, y) = ctx.gfx.drawable_size();

        let mut text = graphics::Text::new(format!("{player_name} wins!"));
        text.set_scale(PxScale::from(24.));

        let rect = text.dimensions(ctx).unwrap();

        canvas.draw(
            &text,
            DrawParam::default().dest(Point2 {
                x: x * 0.5 - rect.w * 0.5,
                y: y * 0.5,
            }),
        );
    }

    fn draw_score(&self, ctx: &mut Context, canvas: &mut Canvas) {
        let (x, _) = ctx.gfx.drawable_size();
        let mut score_text = String::new();

        for player in self.players.iter() {
            writeln!(score_text, "{}: {}", player.name, player.score).unwrap();
        }

        let score_text = graphics::Text::new(score_text);
        let score_rect = score_text.dimensions(ctx).unwrap();

        let draw_param = DrawParam::default().dest(Point2 {
            x: x * 0.5 - score_rect.w * 0.5,
            y: 30.0,
        });

        canvas.draw(&score_text, draw_param);
    }
}

/// Holds the absolute bounds of a Kurve instance
#[derive(Debug, Clone, Copy)]
pub struct ArenaBounds {
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
}

impl ArenaBounds {
    /// Return arena bounds configured from an arbitrary center.
    pub fn new(
        center: Point2<f32>,
        (size_x, size_y): (f32, f32),
        (mul_x, mul_y): (f32, f32),
    ) -> Self {
        let size = (size_x * mul_x, size_y * mul_y);

        let (x_min, x_max) = (center.x - size.0 * 0.5, center.x + size.0 * 0.5);
        let (y_min, y_max) = (center.y - size.1 * 0.5, center.y + size.1 * 0.5);

        Self {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }

    /// Return arena bounds configured from the center of the screen.
    pub fn new_center((size_x, size_y): (f32, f32), (mul_x, mul_y): (f32, f32)) -> Self {
        let size = (size_x * mul_x, size_y * mul_y);

        let center = Point2 {
            x: size_x * 0.5,
            y: size_y * 0.5,
        };

        let (x_min, x_max) = (center.x - size.0 * 0.5, center.x + size.0 * 0.5);
        let (y_min, y_max) = (center.y - size.1 * 0.5, center.y + size.1 * 0.5);

        Self {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }

    /// Return a random point within this arena's bounds
    pub fn random_pos(&self) -> Point2<f32> {
        random_pos((self.x_min, self.x_max), (self.y_min, self.y_max))
    }
}

#[inline]
pub fn check_line_collision(bbox: BoundingBox, line: &Line) -> bool {
    for bp in bbox.iter() {
        for pt in line.iter() {
            if pt.x == bp.x && pt.y == bp.y {
                return true;
            }
        }
    }

    false
}

#[inline]
pub fn check_border_collision(
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
    bbox: BoundingBox,
) -> bool {
    for point in bbox {
        if point.x < x_min || point.x > x_max || point.y < y_min || point.y > y_max {
            return true;
        }
    }

    false
}

enum Collision {
    Min,
    Max,
}

#[inline]
fn check_border_axis_collision(min: f32, max: f32, bbox: [f32; 9]) -> Option<Collision> {
    for point in bbox {
        if point < min {
            return Some(Collision::Min);
        }

        if point > max {
            return Some(Collision::Max);
        }
    }

    None
}

#[inline]
fn random_pos<T>(bounds_x: (T, T), bounds_y: (T, T)) -> Point2<T>
where
    T: SampleUniform + PartialOrd,
{
    Point2 {
        x: rand::thread_rng().gen_range(bounds_x.0..bounds_x.1),
        y: rand::thread_rng().gen_range(bounds_y.0..bounds_y.1),
    }
}

#[inline]
fn random_rot() -> f32 {
    rand::thread_rng().gen_range(0f32..2. * PI)
}
