use std::ffi::c_void;

pub type Identifier = u32;
pub type Real = f32;
pub type Font = *mut c_void;
pub type Image = *mut c_void;

pub const MAX_INSTRUCTIONS: usize = 256 * 1024;
pub const MAX_ROOTS: usize = 32;
pub const MAX_CONTAINERS: usize = 32;
pub const MAX_CLIPS: usize = 32;
pub const MAX_IDENTIFIERS: usize = 32;
pub const MAX_LAYOUTS: usize = 16;
pub const POOL_CAPACITY: usize = 48;
pub const TREE_CAPACITY: usize = 48;
pub const MAX_WIDTHS: usize = 16;
pub const MAX_FORMAT: usize = 127;

pub const CLIP_PARTIAL: i32 = 1;
pub const CLIP_ALL: i32 = 2;

pub const RESULT_ACTIVE: i32 = 1 << 0;
pub const RESULT_SUBMIT: i32 = 1 << 1;
pub const RESULT_CHANGE: i32 = 1 << 2;

pub const ALIGN_CENTER: i32 = 1 << 0;
pub const ALIGN_RIGHT: i32 = 1 << 1;
pub const PASSIVE: i32 = 1 << 2;
pub const NO_FRAME: i32 = 1 << 3;
pub const NO_RESIZE: i32 = 1 << 4;
pub const NO_SCROLL: i32 = 1 << 5;
pub const NO_CLOSE: i32 = 1 << 6;
pub const NO_TITLE: i32 = 1 << 7;
pub const HOLD_FOCUS: i32 = 1 << 8;
pub const AUTO_SIZE: i32 = 1 << 9;
pub const IS_POPUP: i32 = 1 << 10;
pub const IS_CLOSED: i32 = 1 << 11;
pub const IS_EXPANDED: i32 = 1 << 12;
pub const MULTILINE: i32 = 1 << 13;

pub const MOUSE_LEFT: i32 = 1 << 0;
pub const MOUSE_RIGHT: i32 = 1 << 1;
pub const MOUSE_MIDDLE: i32 = 1 << 2;

pub const KEY_SHIFT: i32 = 1 << 0;
pub const KEY_CONTROL: i32 = 1 << 1;
pub const KEY_ALT: i32 = 1 << 2;
pub const KEY_BACKSPACE: i32 = 1 << 3;
pub const KEY_RETURN: i32 = 1 << 4;
pub const KEY_LEFT: i32 = 1 << 5;
pub const KEY_RIGHT: i32 = 1 << 6;
pub const KEY_UP: i32 = 1 << 7;
pub const KEY_DOWN: i32 = 1 << 8;
pub const KEY_DELETE: i32 = 1 << 9;
pub const KEY_A: i32 = 1 << 10;
pub const KEY_C: i32 = 1 << 11;
pub const KEY_V: i32 = 1 << 12;
pub const KEY_X: i32 = 1 << 13;

#[derive(Clone, Copy, Debug, Default)]
pub struct Vector {
    pub x: i32,
    pub y: i32,
}

impl Vector {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rectangle {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    pub fn expand(self, amount: i32) -> Self {
        Self::new(
            self.x - amount,
            self.y - amount,
            self.width + amount * 2,
            self.height + amount * 2,
        )
    }

    pub fn intersect(self, other: Self) -> Self {
        let left = i32::max(self.x, other.x);
        let top = i32::max(self.y, other.y);
        let right = i32::min(self.x + self.width, other.x + other.width);
        let bottom = i32::min(self.y + self.height, other.y + other.height);
        let w = i32::max(right - left, 0);
        let h = i32::max(bottom - top, 0);
        Self::new(left, top, w, h)
    }

    pub fn contains_point(self, point: Vector) -> bool {
        point.x >= self.x
            && point.x < self.x + self.width
            && point.y >= self.y
            && point.y < self.y + self.height
    }

    pub fn unbounded() -> Self {
        Self::new(0, 0, 0x1000000, 0x1000000)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self { red, green, blue, alpha }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorIndex {
    Text = 0,
    Border,
    Window,
    Title,
    Heading,
    Panel,
    Button,
    ButtonHover,
    ButtonFocus,
    Input,
    InputHover,
    InputFocus,
    Scroll,
    Thumb,
    Selection,
    Count,
}

pub const COLOR_COUNT: usize = ColorIndex::Count as usize;

#[derive(Clone, Copy, Debug, Default)]
pub struct Style {
    pub font: Font,
    pub size: Vector,
    pub padding: i32,
    pub spacing: i32,
    pub indent: i32,
    pub title_height: i32,
    pub scroll_size: i32,
    pub thumb_size: i32,
    pub corner_radius: f32,
    pub colors: [Color; COLOR_COUNT],
}

#[derive(Clone, Copy, Debug)]
pub struct PoolEntry {
    pub identifier: Identifier,
    pub update: u32,
}

impl Default for PoolEntry {
    fn default() -> Self {
        Self {
            identifier: 0,
            update: 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Instruction {
    Jump {
        target: usize,
    },
    Clip {
        bounds: Rectangle,
    },
    Rectangle {
        bounds: Rectangle,
        radius: f32,
        mode: u32,
        color: Color,
    },
    Text {
        font: Font,
        position: Vector,
        color: Color,
        string_offset: usize,
        string_length: usize,
    },
    Icon {
        identifier: i32,
        bounds: Rectangle,
        color: Color,
    },
    Image {
        source: Image,
        bounds: Rectangle,
        tint: Color,
    },
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Layout {
    pub body: Rectangle,
    pub next: Rectangle,
    pub position: Vector,
    pub size: Vector,
    pub maximum_size: Vector,
    pub widths: [i32; MAX_WIDTHS],
    pub items: i32,
    pub item_index: i32,
    pub next_row: i32,
    pub next_type: i32,
    pub indent: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct Container {
    pub head: usize,
    pub tail: usize,
    pub bounds: Rectangle,
    pub body: Rectangle,
    pub content: Vector,
    pub scroll: Vector,
    pub depth: i32,
    pub open: bool,
}

impl Default for Container {
    fn default() -> Self {
        Self {
            head: 0,
            tail: 0,
            bounds: Rectangle::default(),
            body: Rectangle::default(),
            content: Vector::default(),
            scroll: Vector::default(),
            depth: 0,
            open: false,
        }
    }
}