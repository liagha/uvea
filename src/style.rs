use crate::types::{Color, ColorSlot, Font, Vec2, COLOR_COUNT};

#[derive(Debug, Clone)]
pub struct Style {
    pub font: Font,
    pub size: Vec2,
    pub padding: i32,
    pub spacing: i32,
    pub indent: i32,
    pub title_height: i32,
    pub scroll_size: i32,
    pub thumb_size: i32,
    pub corner_radius: f32,
    pub colors: [Color; COLOR_COUNT],
}

impl Style {
    pub fn color(&self, slot: ColorSlot) -> Color {
        self.colors[slot as usize]
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            font: 0,
            size: Vec2::new(68, 10),
            padding: 5,
            spacing: 4,
            indent: 24,
            title_height: 24,
            scroll_size: 12,
            thumb_size: 8,
            corner_radius: 0.0,
            colors: [
                Color::rgba(230, 230, 230, 255),
                Color::rgba(25, 25, 25, 255),
                Color::rgba(50, 50, 50, 255),
                Color::rgba(25, 25, 25, 255),
                Color::rgba(240, 240, 240, 255),
                Color::rgba(0, 0, 0, 0),
                Color::rgba(75, 75, 75, 255),
                Color::rgba(95, 95, 95, 255),
                Color::rgba(115, 115, 115, 255),
                Color::rgba(30, 30, 30, 255),
                Color::rgba(35, 35, 35, 255),
                Color::rgba(40, 40, 40, 255),
                Color::rgba(43, 43, 43, 255),
                Color::rgba(30, 30, 30, 255),
                Color::rgba(80, 120, 200, 150),
            ],
        }
    }
}