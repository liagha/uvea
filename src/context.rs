use crate::{
    draw::DrawList,
    layout::Layout,
    style::Style,
    types::*,
};

const POOL: usize = 48;
const TREE: usize = 48;

#[derive(Debug, Clone, Default)]
pub struct Container {
    pub bounds: Rect,
    pub body: Rect,
    pub content: Vec2,
    pub scroll: Vec2,
    pub depth: i32,
    pub open: bool,
}

#[derive(Debug, Default)]
struct PoolEntry {
    id: Id,
    frame: u32,
}

pub struct Context {
    pub style: Style,
    pub draws: DrawList,

    pub hover: Id,
    pub focus: Id,
    pub focus_updated: bool,

    pub text_focus: Id,
    pub text_scroll: i32,
    pub cursor: usize,
    pub selection: usize,
    pub number_edit: Id,
    pub number_buf: String,
    pub scroll: Vec2,

    pub frame: u32,
    pub last_id: Id,
    pub last_bounds: Rect,
    pub last_depth: i32,

    pub mouse: Vec2,
    pub last_mouse: Vec2,
    pub delta: Vec2,
    pub scroll_delta: Vec2,
    pub mouse_down: u32,
    pub mouse_pressed: u32,
    pub key_down: u32,
    pub key_pressed: u32,
    pub input: String,
    pub clipboard: String,

    pub hover_root: Option<usize>,
    pub next_root: Option<usize>,
    pub scroll_target: Option<usize>,

    containers: Vec<Container>,
    container_pool: Vec<PoolEntry>,
    tree_pool: Vec<PoolEntry>,
    id_stack: Vec<Id>,
    layout_stack: Vec<Layout>,
    container_stack: Vec<usize>,
    clip_stack: Vec<Rect>,
    roots: Vec<usize>,
}

impl Context {
    pub fn new() -> Self {
        let mut ctx = Self {
            style: Style::default(),
            draws: DrawList::default(),
            hover: 0,
            focus: 0,
            focus_updated: false,
            text_focus: 0,
            text_scroll: 0,
            cursor: 0,
            selection: 0,
            number_edit: 0,
            number_buf: String::new(),
            scroll: Vec2::default(),
            frame: 0,
            last_id: 0,
            last_bounds: Rect::default(),
            last_depth: 0,
            mouse: Vec2::default(),
            last_mouse: Vec2::default(),
            delta: Vec2::default(),
            scroll_delta: Vec2::default(),
            mouse_down: 0,
            mouse_pressed: 0,
            key_down: 0,
            key_pressed: 0,
            input: String::new(),
            clipboard: String::new(),
            hover_root: None,
            next_root: None,
            scroll_target: None,
            containers: Vec::new(),
            container_pool: (0..POOL).map(|_| PoolEntry::default()).collect(),
            tree_pool: (0..TREE).map(|_| PoolEntry::default()).collect(),
            id_stack: Vec::new(),
            layout_stack: Vec::new(),
            container_stack: Vec::new(),
            clip_stack: Vec::new(),
            roots: Vec::new(),
        };
        ctx.containers.resize_with(POOL, Container::default);
        ctx
    }

    pub fn begin_frame(&mut self) {
        self.draws.clear();
        self.roots.clear();
        self.scroll_target = None;
        self.hover_root = self.next_root;
        self.next_root = None;
        self.delta = Vec2::new(
            self.mouse.x - self.last_mouse.x,
            self.mouse.y - self.last_mouse.y,
        );
        self.frame += 1;
    }

    pub fn end_frame(&mut self) {
        assert!(self.container_stack.is_empty());
        assert!(self.clip_stack.is_empty());
        assert!(self.id_stack.is_empty());
        assert!(self.layout_stack.is_empty());

        if let Some(target) = self.scroll_target {
            self.containers[target].scroll.x += self.scroll_delta.x;
            self.containers[target].scroll.y += self.scroll_delta.y;
        }

        if !self.focus_updated {
            self.focus = 0;
        }
        self.focus_updated = false;

        self.key_pressed = 0;
        self.input.clear();
        self.mouse_pressed = 0;
        self.scroll_delta = Vec2::default();
        self.last_mouse = self.mouse;
    }

    pub fn id(&mut self, data: &[u8]) -> Id {
        let base = self.id_stack.last().copied().unwrap_or(2166136261);
        let result = data.iter().fold(base, |acc, &b| (acc ^ b as u32).wrapping_mul(16777619));
        self.last_id = result;
        result
    }

    pub fn id_str(&mut self, s: &str) -> Id {
        self.id(s.as_bytes())
    }

    pub fn push_id(&mut self, data: &[u8]) {
        let id = self.id(data);
        self.id_stack.push(id);
    }

    pub fn pop_id(&mut self) {
        self.id_stack.pop();
    }

    pub fn push_clip(&mut self, bounds: Rect) {
        let last = self.clip();
        self.clip_stack.push(bounds.intersect(last));
    }

    pub fn pop_clip(&mut self) {
        self.clip_stack.pop();
    }

    pub fn clip(&self) -> Rect {
        self.clip_stack.last().copied().unwrap_or(Rect::unbounded())
    }

    pub fn check_clip(&self, bounds: Rect) -> Clip {
        let clip = self.clip();
        if bounds.x > clip.x + clip.w
            || bounds.x + bounds.w < clip.x
            || bounds.y > clip.y + clip.h
            || bounds.y + bounds.h < clip.y
        {
            return Clip::All;
        }
        if bounds.x >= clip.x
            && bounds.x + bounds.w <= clip.x + clip.w
            && bounds.y >= clip.y
            && bounds.y + bounds.h <= clip.y + clip.h
        {
            return Clip::None;
        }
        Clip::Partial
    }

    pub fn set_focus(&mut self, id: Id) {
        self.focus = id;
        self.focus_updated = true;
        if id == 0 {
            self.text_focus = 0;
            self.text_scroll = 0;
        }
    }

    pub fn layout(&mut self) -> &mut Layout {
        self.layout_stack.last_mut().expect("no layout")
    }

    pub fn push_layout(&mut self, body: Rect, scroll: Vec2) {
        self.layout_stack.push(Layout::new(body, scroll));
    }

    pub fn pop_layout(&mut self) {
        self.layout_stack.pop();
    }

    pub fn next_bounds(&mut self) -> Rect {
        let spacing = self.style.spacing;
        let size = self.style.size;
        self.layout().next(size, spacing)
    }

    pub fn current_container(&self) -> &Container {
        let idx = *self.container_stack.last().expect("no container");
        &self.containers[idx]
    }

    pub fn current_container_mut(&mut self) -> &mut Container {
        let idx = *self.container_stack.last().expect("no container");
        &mut self.containers[idx]
    }

    pub fn context_containers(&mut self) -> &mut Vec<Container> {
        &mut self.containers
    }

    pub fn fetch_container(&mut self, id: Id, closed: bool) -> Option<usize> {
        let found = self.container_pool.iter().position(|e| e.id == id);
        if let Some(idx) = found {
            if self.containers[idx].open || !closed {
                self.container_pool[idx].frame = self.frame;
            }
            return Some(idx);
        }
        if closed {
            return None;
        }
        let slot = self
            .container_pool
            .iter()
            .enumerate()
            .min_by_key(|(_, e)| e.frame)
            .map(|(i, _)| i)
            .unwrap();
        self.container_pool[slot] = PoolEntry { id, frame: self.frame };
        self.containers[slot] = Container::default();
        self.containers[slot].open = true;
        self.bring_front(slot);
        Some(slot)
    }

    pub fn bring_front(&mut self, idx: usize) {
        self.last_depth += 1;
        self.containers[idx].depth = self.last_depth;
    }

    pub fn fetch_tree(&mut self, id: Id) -> bool {
        self.tree_pool.iter().any(|e| e.id == id)
    }

    pub fn activate_tree(&mut self, id: Id) {
        let slot = self
            .tree_pool
            .iter()
            .enumerate()
            .min_by_key(|(_, e)| e.frame)
            .map(|(i, _)| i)
            .unwrap();
        self.tree_pool[slot] = PoolEntry { id, frame: self.frame };
    }

    pub fn deactivate_tree(&mut self, id: Id) {
        if let Some(slot) = self.tree_pool.iter().position(|e| e.id == id) {
            self.tree_pool[slot] = PoolEntry::default();
        }
    }

    pub fn update_tree(&mut self, id: Id) {
        if let Some(slot) = self.tree_pool.iter().position(|e| e.id == id) {
            self.tree_pool[slot].frame = self.frame;
        }
    }

    pub fn push_container(&mut self, idx: usize) {
        self.draws.clip(Rect::unbounded());
        self.container_stack.push(idx);
        self.roots.push(idx);

        let mouse = self.mouse;
        let depth = self.containers[idx].depth;
        if self.containers[idx].bounds.contains(mouse) {
            let deeper = self.next_root.map_or(true, |r| depth > self.containers[r].depth);
            if deeper {
                self.next_root = Some(idx);
            }
        }
    }

    pub fn push_container_raw(&mut self, idx: usize) {
        self.container_stack.push(idx);
    }

    pub fn pop_container(&mut self) {
        let idx = self.container_stack.pop().expect("unbalanced pop");
        let layout = self.layout_stack.pop().expect("unbalanced pop");
        self.containers[idx].content = layout.content_size();
        self.id_stack.pop();
        self.clip_stack.pop();
    }

    pub fn hovering(&self) -> bool {
        let mut index = self.container_stack.len();
        while index > 0 {
            index -= 1;
            let idx = self.container_stack[index];
            if Some(idx) == self.hover_root {
                return true;
            }
            if self.containers[idx].open {
                break;
            }
        }
        false
    }
}