use crate::{context::Context, types::*};

impl Context {
    pub fn mouse_move(&mut self, x: i32, y: i32) {
        self.mouse = Vec2::new(x, y);
    }

    pub fn mouse_down(&mut self, x: i32, y: i32, button: u32) {
        self.mouse = Vec2::new(x, y);
        self.mouse_down |= button;
        self.mouse_pressed |= button;
    }

    pub fn mouse_up(&mut self, x: i32, y: i32, button: u32) {
        self.mouse = Vec2::new(x, y);
        self.mouse_down &= !button;
    }

    pub fn scroll(&mut self, dx: i32, dy: i32) {
        self.scroll_delta.x += dx;
        self.scroll_delta.y += dy;
    }

    pub fn key_down(&mut self, key: u32) {
        self.key_pressed |= key;
        self.key_down |= key;
    }

    pub fn key_up(&mut self, key: u32) {
        self.key_down &= !key;
    }

    pub fn text_input(&mut self, text: &str) {
        self.input.push_str(text);
    }

    pub fn set_clipboard(&mut self, text: impl Into<String>) {
        self.clipboard = text.into();
    }

    pub fn get_clipboard(&self) -> &str {
        &self.clipboard
    }
}