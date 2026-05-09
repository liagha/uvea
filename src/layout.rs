use crate::types::{Rect, Vec2};

const MAX_WIDTHS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NextType {
    None,
    Relative,
    Absolute,
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub body: Rect,
    position: Vec2,
    pub size: Vec2,
    pub maximum: Vec2,
    widths: [i32; MAX_WIDTHS],
    items: usize,
    item_index: usize,
    next_row: i32,
    next: Rect,
    next_type: NextType,
    pub indent: i32,
}

impl Layout {
    pub fn new(body: Rect, scroll: Vec2) -> Self {
        let shifted = Rect::new(
            body.x - scroll.x,
            body.y - scroll.y,
            body.w,
            body.h,
        );
        let mut layout = Self {
            body: shifted,
            position: Vec2::default(),
            size: Vec2::default(),
            maximum: Vec2::new(-0x1000000, -0x1000000),
            widths: [0; MAX_WIDTHS],
            items: 0,
            item_index: 0,
            next_row: 0,
            next: Rect::default(),
            next_type: NextType::None,
            indent: 0,
        };
        layout.row(1, &[0], 0);
        layout
    }

    pub fn row(&mut self, count: usize, widths: &[i32], height: i32) {
        assert!(count <= MAX_WIDTHS);
        self.widths[..count].copy_from_slice(&widths[..count.min(widths.len())]);
        self.items = count;
        self.position = Vec2::new(self.indent, self.next_row);
        self.size.y = height;
        self.item_index = 0;
    }

    pub fn set_next(&mut self, bounds: Rect, absolute: bool) {
        self.next = bounds;
        self.next_type = if absolute { NextType::Absolute } else { NextType::Relative };
    }

    pub fn next(&mut self, default_size: Vec2, spacing: i32) -> Rect {
        if self.next_type != NextType::None {
            let kind = self.next_type;
            self.next_type = NextType::None;
            if kind == NextType::Absolute {
                return self.next;
            }
            let mut result = self.next;
            result.x += self.body.x;
            result.y += self.body.y;
            self.track(result, spacing);
            return result;
        }

        if self.item_index == self.items {
            self.row(self.items, &[], self.size.y);
        }

        let mut result = Rect::new(
            self.position.x,
            self.position.y,
            if self.items > 0 { self.widths[self.item_index] } else { self.size.x },
            self.size.y,
        );

        if result.w == 0 { result.w = default_size.x + spacing * 2; }
        if result.h == 0 { result.h = default_size.y + spacing * 2; }
        if result.w < 0 { result.w += self.body.w - result.x + 1; }
        if result.h < 0 { result.h += self.body.h - result.y + 1; }

        self.item_index += 1;
        result.x += self.body.x;
        result.y += self.body.y;
        self.track(result, spacing);
        result
    }

    fn track(&mut self, result: Rect, spacing: i32) {
        self.position.x = result.x - self.body.x + result.w + spacing;
        self.next_row = self.next_row.max(result.y - self.body.y + result.h + spacing);
        self.maximum.x = self.maximum.x.max(result.x + result.w);
        self.maximum.y = self.maximum.y.max(result.y + result.h);
    }

    pub fn content_size(&self) -> Vec2 {
        Vec2::new(
            self.maximum.x - self.body.x,
            self.maximum.y - self.body.y,
        )
    }
}