use ggez::graphics::{self, Canvas, Color, DrawParam, Drawable, PxScale};
use ggez::mint::Point2;
use ggez::{Context, GameResult};
use std::fmt::Debug;

#[derive(Debug)]
pub struct MainMenu {
    pub items: [MainMenuItem; 1],
    pub selected: usize,
}

impl MainMenu {
    pub fn new() -> Self {
        Self {
            items: [MainMenuItem::PlayButton { size: (200., 60.) }],
            selected: 0,
        }
    }

    pub fn draw(&self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
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
