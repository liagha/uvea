// src/context.rs
use arrayvec::ArrayVec;
use crate::types::*;
use crate::stack;
use crate::pool::Pool;

fn hash_identifier(current: Identifier, data: &[u8]) -> Identifier {
    let mut result = current;
    for byte in data {
        result = (result ^ *byte as Identifier).wrapping_mul(16777619);
    }
    result
}

pub struct Context {
    pub text_width: Option<fn(Font, &[u8]) -> i32>,
    pub text_height: Option<fn(Font) -> i32>,
    pub draw_frame: fn(&mut Context, Rectangle, ColorIndex),
    pub set_clipboard: Option<fn(&mut Context, &str)>,
    pub get_clipboard: Option<fn(&mut Context) -> String>,

    pub standard_style: Style,
    pub style: *const Style,
    pub hover: Identifier,
    pub focus: Identifier,
    pub last_identifier: Identifier,
    pub last_bounds: Rectangle,
    pub last_depth: i32,
    pub focus_updated: bool,
    pub text_focus: Identifier,
    pub text_scroll: i32,
    pub frame: u32,
    pub hover_root: Option<usize>,
    pub next_root: Option<usize>,
    pub scroll_target: Option<usize>,
    pub number_buffer: [u8; MAX_FORMAT],
    pub number_edit: Identifier,
    pub cursor: usize,
    pub selection: usize,
    pub scroll: Vector,

    pub instructions: Vec<Instruction>,
    pub roots: ArrayVec<usize, MAX_ROOTS>,
    pub containers_stack: ArrayVec<usize, MAX_CONTAINERS>,
    pub clips: ArrayVec<Rectangle, MAX_CLIPS>,
    pub identifiers: ArrayVec<Identifier, MAX_IDENTIFIERS>,
    pub layouts: ArrayVec<Layout, MAX_LAYOUTS>,
    pub text_buffer: Vec<u8>,

    pub container_pool: Pool<Container, POOL_CAPACITY>,
    pub tree_pool: Pool<PoolEntry, TREE_CAPACITY>,

    pub mouse: Vector,
    pub last_mouse: Vector,
    pub mouse_delta: Vector,
    pub scroll_delta: Vector,
    pub mouse_down: i32,
    pub mouse_pressed: i32,
    pub key_down: i32,
    pub key_pressed: i32,
    pub input: String,
}

impl Context {
    pub fn new(
        text_width: fn(Font, &[u8]) -> i32,
        text_height: fn(Font) -> i32,
    ) -> Self {
        let standard_style = Style {
            font: std::ptr::null_mut(),
            size: Vector::new(68, 10),
            padding: 5,
            spacing: 4,
            indent: 24,
            title_height: 24,
            scroll_size: 12,
            thumb_size: 8,
            corner_radius: 0.0,
            colors: [
                Color::new(230, 230, 230, 255),
                Color::new(25, 25, 25, 255),
                Color::new(50, 50, 50, 255),
                Color::new(25, 25, 25, 255),
                Color::new(240, 240, 240, 255),
                Color::new(0, 0, 0, 0),
                Color::new(75, 75, 75, 255),
                Color::new(95, 95, 95, 255),
                Color::new(115, 115, 115, 255),
                Color::new(30, 30, 30, 255),
                Color::new(35, 35, 35, 255),
                Color::new(40, 40, 40, 255),
                Color::new(43, 43, 43, 255),
                Color::new(30, 30, 30, 255),
                Color::new(80, 120, 200, 150),
            ],
        };

        let mut result = Self {
            text_width: Some(text_width),
            text_height: Some(text_height),
            draw_frame: Context::default_draw_frame,
            set_clipboard: None,
            get_clipboard: None,
            standard_style,
            style: std::ptr::null(),
            hover: 0,
            focus: 0,
            last_identifier: 0,
            last_bounds: Rectangle::default(),
            last_depth: 0,
            focus_updated: false,
            text_focus: 0,
            text_scroll: 0,
            frame: 0,
            hover_root: None,
            next_root: None,
            scroll_target: None,
            number_buffer: [0u8; MAX_FORMAT],
            number_edit: 0,
            cursor: 0,
            selection: 0,
            scroll: Vector::default(),
            instructions: Vec::with_capacity(MAX_INSTRUCTIONS),
            roots: ArrayVec::new(),
            containers_stack: ArrayVec::new(),
            clips: ArrayVec::new(),
            identifiers: ArrayVec::new(),
            layouts: ArrayVec::new(),
            text_buffer: Vec::with_capacity(32768),
            container_pool: Pool::new(Container::default()),
            tree_pool: Pool::new(PoolEntry::default()),
            mouse: Vector::default(),
            last_mouse: Vector::default(),
            mouse_delta: Vector::default(),
            scroll_delta: Vector::default(),
            mouse_down: 0,
            mouse_pressed: 0,
            key_down: 0,
            key_pressed: 0,
            input: String::new(),
        };
        result.style = &result.standard_style as *const Style;
        result
    }

    pub fn begin_frame(&mut self) {
        assert!(self.text_width.is_some() && self.text_height.is_some());
        self.instructions.clear();
        self.roots.clear();
        self.text_buffer.clear();
        self.scroll_target = None;
        self.hover_root = self.next_root;
        self.next_root = None;
        self.mouse_delta.x = self.mouse.x - self.last_mouse.x;
        self.mouse_delta.y = self.mouse.y - self.last_mouse.y;
        self.frame += 1;
    }

    pub fn end_frame(&mut self) {
        assert!(self.containers_stack.is_empty());
        assert!(self.clips.is_empty());
        assert!(self.identifiers.is_empty());
        assert!(self.layouts.is_empty());

        if let Some(target) = self.scroll_target {
            let container = &mut self.container_pool.items[target];
            container.scroll.x += self.scroll_delta.x;
            container.scroll.y += self.scroll_delta.y;
        }

        if !self.focus_updated {
            self.focus = 0;
        }
        self.focus_updated = false;

        if self.mouse_pressed != 0 {
            if let (Some(next), Some(_)) = (self.next_root, self.hover_root) {
                if self.container_pool.items[next].depth < self.last_depth
                    && self.container_pool.items[next].depth >= 0
                {
                    self.bring_front(next);
                }
            }
        }

        self.key_pressed = 0;
        self.input.clear();
        self.mouse_pressed = 0;
        self.scroll_delta = Vector::new(0, 0);
        self.last_mouse = self.mouse;

        self.roots.sort_by(|a, b| {
            self.container_pool.items[*a]
                .depth
                .cmp(&self.container_pool.items[*b].depth)
        });

        let count = self.roots.len();
        if count == 0 {
            return;
        }

        let head_first = {
            let first_idx = self.roots[0];
            self.container_pool.items[first_idx].head
        };
        self.instructions[0] = Instruction::Jump { target: head_first + 1 };

        let len = self.instructions.len();

        let mut heads = Vec::with_capacity(count);
        let mut tails = Vec::with_capacity(count);
        for &idx in &self.roots {
            let container = &self.container_pool.items[idx];
            heads.push(container.head);
            tails.push(container.tail);
        }

        for i in 1..count {
            let prev_tail = tails[i - 1];
            let head = heads[i];
            if let Instruction::Jump { target } = &mut self.instructions[prev_tail] {
                *target = head + 1;
            }
        }

        let last_tail = tails[count - 1];
        if let Instruction::Jump { target } = &mut self.instructions[last_tail] {
            *target = len;
        }
    }

    pub fn set_focus(&mut self, identifier: Identifier) {
        self.focus = identifier;
        self.focus_updated = true;
        if identifier == 0 {
            self.text_focus = 0;
            self.text_scroll = 0;
        }
    }

    pub fn get_identifier(&mut self, data: &[u8]) -> Identifier {
        let base = if self.identifiers.is_empty() {
            2166136261
        } else {
            self.identifiers[self.identifiers.len() - 1]
        };
        let result = hash_identifier(base, data);
        self.last_identifier = result;
        result
    }

    pub fn push_identifier(&mut self, data: &[u8]) {
        let id = self.get_identifier(data);
        stack::push(&mut self.identifiers, id);
    }

    pub fn pop_identifier(&mut self) {
        stack::pop(&mut self.identifiers);
    }

    pub fn push_clip(&mut self, bounds: Rectangle) {
        let last = self.current_clip();
        let clipped = bounds.intersect(last);
        stack::push(&mut self.clips, clipped);
    }

    pub fn pop_clip(&mut self) {
        stack::pop(&mut self.clips);
    }

    pub fn current_clip(&self) -> Rectangle {
        assert!(!self.clips.is_empty());
        self.clips[self.clips.len() - 1]
    }

    pub fn check_clip(&self, bounds: Rectangle) -> i32 {
        let clip = self.current_clip();
        if bounds.x > clip.x + clip.width || bounds.x + bounds.width < clip.x ||
            bounds.y > clip.y + clip.height || bounds.y + bounds.height < clip.y {
            return CLIP_ALL;
        }
        if bounds.x >= clip.x && bounds.x + bounds.width <= clip.x + clip.width &&
            bounds.y >= clip.y && bounds.y + bounds.height <= clip.y + clip.height {
            return 0;
        }
        CLIP_PARTIAL
    }

    pub fn current_container_index(&self) -> usize {
        *self.containers_stack.last().expect("container stack empty")
    }

    pub fn fetch_container(
        &mut self,
        identifier: Identifier,
        options: i32,
    ) -> Option<usize> {
        if let Some(idx) = self.container_pool.get_index(identifier) {
            self.container_pool.entries[idx].update = self.frame;
            let container = &self.container_pool.items[idx];
            if container.open || options & IS_CLOSED == 0 {
                return Some(idx);
            }
            return None;
        }
        if options & IS_CLOSED != 0 {
            return None;
        }
        let (idx, container) = self.container_pool.get_or_insert(identifier, self.frame);
        *container = Container::default();
        container.open = true;
        self.bring_front(idx);
        Some(idx)
    }

    pub fn find_container(&mut self, name: &str) -> Option<usize> {
        let identifier = self.get_identifier(name.as_bytes());
        self.fetch_container(identifier, 0)
    }

    pub fn bring_front(&mut self, index: usize) {
        self.last_depth += 1;
        self.container_pool.items[index].depth = self.last_depth;
    }

    pub fn input_mouse(&mut self, x: i32, y: i32) {
        self.mouse = Vector::new(x, y);
    }

    pub fn input_down(&mut self, x: i32, y: i32, button: i32) {
        self.input_mouse(x, y);
        self.mouse_down |= button;
        self.mouse_pressed |= button;
    }

    pub fn input_up(&mut self, x: i32, y: i32, button: i32) {
        self.input_mouse(x, y);
        self.mouse_down &= !button;
    }

    pub fn input_scroll(&mut self, x: i32, y: i32) {
        self.scroll_delta.x += x;
        self.scroll_delta.y += y;
    }

    pub fn input_key(&mut self, key: i32) {
        self.key_pressed |= key;
        self.key_down |= key;
    }

    pub fn input_keyup(&mut self, key: i32) {
        self.key_down &= !key;
    }

    pub fn input_text(&mut self, text: &str) {
        self.input.push_str(text);
    }

    fn default_draw_frame(context: &mut Context, bounds: Rectangle, color_index: ColorIndex) {
        let style = unsafe { &*context.style };
        let color = style.colors[color_index as usize];
        if style.corner_radius > 0.0 {
            crate::draw::draw_rounded_rectangle(context, bounds, style.corner_radius, color);
        } else {
            crate::draw::draw_rectangle(context, bounds, color);
        }
        if color_index == ColorIndex::Scroll || color_index == ColorIndex::Thumb || color_index == ColorIndex::Title {
            return;
        }
        if style.colors[ColorIndex::Border as usize].alpha != 0 {
            if style.corner_radius > 0.0 {
                crate::draw::draw_rounded_box(context, bounds, style.corner_radius, style.colors[ColorIndex::Border as usize]);
            } else {
                crate::draw::draw_box(context, bounds.expand(1), style.colors[ColorIndex::Border as usize]);
            }
        }
    }
}