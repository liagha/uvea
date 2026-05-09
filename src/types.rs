#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Vec2 {
    pub x: i32,
    pub y: i32,
}

impl Vec2 {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    pub fn expand(self, amount: i32) -> Self {
        Self::new(
            self.x - amount,
            self.y - amount,
            self.w + amount * 2,
            self.h + amount * 2,
        )
    }

    pub fn intersect(self, other: Self) -> Self {
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = (self.x + self.w).min(other.x + other.w).max(left);
        let bottom = (self.y + self.h).min(other.y + other.h).max(top);
        Self::new(left, top, right - left, bottom - top)
    }

    pub fn contains(self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x < self.x + self.w
            && point.y >= self.y
            && point.y < self.y + self.h
    }

    pub fn unbounded() -> Self {
        Self::new(0, 0, 0x1000000, 0x1000000)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);
    pub const WHITE: Self = Self::rgba(255, 255, 255, 255);
}

pub type Id = u32;
pub type Font = usize;
pub type Image = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mouse {
    Left = 1 << 0,
    Right = 1 << 1,
    Middle = 1 << 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Shift = 1 << 0,
    Control = 1 << 1,
    Alt = 1 << 2,
    Backspace = 1 << 3,
    Return = 1 << 4,
    Left = 1 << 5,
    Right = 1 << 6,
    Up = 1 << 7,
    Down = 1 << 8,
    Delete = 1 << 9,
    A = 1 << 10,
    C = 1 << 11,
    V = 1 << 12,
    X = 1 << 13,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Result {
    Active = 1 << 0,
    Submit = 1 << 1,
    Change = 1 << 2,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Options: u32 {
        const ALIGN_CENTER  = 1 << 0;
        const ALIGN_RIGHT   = 1 << 1;
        const PASSIVE       = 1 << 2;
        const NO_FRAME      = 1 << 3;
        const NO_RESIZE     = 1 << 4;
        const NO_SCROLL     = 1 << 5;
        const NO_CLOSE      = 1 << 6;
        const NO_TITLE      = 1 << 7;
        const HOLD_FOCUS    = 1 << 8;
        const AUTO_SIZE     = 1 << 9;
        const IS_POPUP      = 1 << 10;
        const IS_CLOSED     = 1 << 11;
        const IS_EXPANDED   = 1 << 12;
        const MULTILINE     = 1 << 13;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Response: u32 {
        const ACTIVE  = 1 << 0;
        const SUBMIT  = 1 << 1;
        const CHANGE  = 1 << 2;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Clip {
    None,
    Partial,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Close = 1,
    Check,
    Collapsed,
    Expanded,
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSlot {
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
}

pub const COLOR_COUNT: usize = 15;