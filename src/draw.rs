use crate::types::{Color, Font, Icon, Image, Rect, Vec2};

#[derive(Debug, Clone)]
pub enum Command {
    Clip(Rect),
    Rect { bounds: Rect, color: Color, radius: f32, outline: bool },
    Text { font: Font, text: String, position: Vec2, color: Color },
    Icon { kind: Icon, bounds: Rect, color: Color },
    Image { source: Image, bounds: Rect, tint: Color },
}

#[derive(Debug, Default)]
pub struct DrawList {
    commands: Vec<Command>,
}

impl DrawList {
    pub fn push(&mut self, cmd: Command) {
        self.commands.push(cmd);
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &Command> {
        self.commands.iter()
    }

    pub fn clip(&mut self, bounds: Rect) {
        self.commands.push(Command::Clip(bounds));
    }

    pub fn rect(&mut self, bounds: Rect, color: Color) {
        if bounds.w > 0 && bounds.h > 0 {
            self.commands.push(Command::Rect {
                bounds,
                color,
                radius: 0.0,
                outline: false,
            });
        }
    }

    pub fn rounded_rect(&mut self, bounds: Rect, color: Color, radius: f32) {
        if bounds.w > 0 && bounds.h > 0 {
            self.commands.push(Command::Rect {
                bounds,
                color,
                radius,
                outline: false,
            });
        }
    }

    pub fn rounded_outline(&mut self, bounds: Rect, color: Color, radius: f32) {
        if bounds.w > 0 && bounds.h > 0 {
            self.commands.push(Command::Rect {
                bounds,
                color,
                radius,
                outline: true,
            });
        }
    }

    pub fn border(&mut self, bounds: Rect, color: Color) {
        let e = bounds.expand(1);
        self.rect(Rect::new(e.x + 1, e.y, e.w - 2, 1), color);
        self.rect(Rect::new(e.x + 1, e.y + e.h - 1, e.w - 2, 1), color);
        self.rect(Rect::new(e.x, e.y, 1, e.h), color);
        self.rect(Rect::new(e.x + e.w - 1, e.y, 1, e.h), color);
    }

    pub fn text(&mut self, font: Font, text: impl Into<String>, position: Vec2, color: Color) {
        self.commands.push(Command::Text {
            font,
            text: text.into(),
            position,
            color,
        });
    }

    pub fn icon(&mut self, kind: Icon, bounds: Rect, color: Color) {
        self.commands.push(Command::Icon { kind, bounds, color });
    }

    pub fn image(&mut self, source: Image, bounds: Rect, tint: Color) {
        self.commands.push(Command::Image { source, bounds, tint });
    }
}