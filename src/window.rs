// src/window.rs

use crate::context::Context;
use crate::draw::*;
use crate::types::*;
use std::cmp;

impl Context {
    fn draw_header_control(
        &mut self,
        label: &str,
        tree: bool,
        options: i32,
    ) -> i32 {
        let identifier = self.get_identifier(label.as_bytes());
        let pool_idx = self.tree_pool.get_index(identifier);
        let width = -1;
        self.layout_row(1, Some(&[width]), 0);

        let mut active = pool_idx.is_some();
        let expanded = if options & IS_EXPANDED != 0 { !active } else { active };
        let bounds = self.next_bounds();
        self.update_control(identifier, bounds, 0);

        active ^= self.mouse_pressed == MOUSE_LEFT && self.focus == identifier;

        if let Some(idx) = pool_idx {
            if active {
                self.tree_pool.entries[idx].update = self.frame;
            } else {
                self.tree_pool.entries[idx] = PoolEntry::default();
            }
        } else if active {
            let entry = self.tree_pool.get_or_insert(identifier, self.frame).1;
            *entry = PoolEntry::default();
        }

        if tree {
            if self.hover == identifier {
                let draw = self.draw_frame;
                draw(self, bounds, ColorIndex::ButtonHover);
            }
        } else {
            self.control_frame(identifier, bounds, ColorIndex::Button, 0);
        }

        let icon = if expanded { 4 } else { 3 };
        let style = unsafe { &*self.style };
        draw_icon(
            self,
            icon,
            Rectangle::new(bounds.x, bounds.y, bounds.height, bounds.height),
            style.colors[ColorIndex::Text as usize],
        );
        let mut label_bounds = bounds;
        label_bounds.x += bounds.height - style.padding;
        label_bounds.width -= bounds.height - style.padding;
        self.control_text(label, label_bounds, ColorIndex::Text, 0);

        if expanded { RESULT_ACTIVE } else { 0 }
    }

    pub fn draw_header(&mut self, label: &str, options: i32) -> i32 {
        self.draw_header_control(label, false, options)
    }

    pub fn push_tree(&mut self, label: &str, options: i32) -> i32 {
        let result = self.draw_header_control(label, true, options);
        if result & RESULT_ACTIVE != 0 {
            self.current_layout_mut().indent += unsafe { &*self.style }.indent;
            self.identifiers.push(self.last_identifier);
        }
        result
    }

    pub fn pop_tree(&mut self) {
        self.current_layout_mut().indent -= unsafe { &*self.style }.indent;
        self.pop_identifier();
    }

    pub fn push_window(
        &mut self,
        title: &str,
        bounds: Rectangle,
        options: i32,
    ) -> i32 {
        let identifier = self.get_identifier(title.as_bytes());
        let container_idx = match self.fetch_container(identifier, options) {
            Some(idx) if self.container_pool.items[idx].open => idx,
            _ => return 0,
        };
        self.identifiers.push(identifier);

        {
            let container = &mut self.container_pool.items[container_idx];
            if container.bounds.width == 0 {
                container.bounds = bounds;
            }
        }

        self.begin_root(container_idx);

        let style = unsafe { &*self.style };

        if options & NO_FRAME == 0 {
            let draw = self.draw_frame;
            let container_bounds = self.container_pool.items[container_idx].bounds;
            draw(self, container_bounds, ColorIndex::Window);
        }

        if options & NO_TITLE == 0 {
            let container_bounds = self.container_pool.items[container_idx].bounds;
            let mut title_bounds = Rectangle::new(
                container_bounds.x, container_bounds.y,
                container_bounds.width, style.title_height,
            );
            let draw = self.draw_frame;
            draw(self, title_bounds, ColorIndex::Title);

            let title_id = self.get_identifier("!title".as_bytes());
            self.update_control(title_id, title_bounds, options);
            self.control_text(title, title_bounds, ColorIndex::Heading, options);
            if title_id == self.focus && self.mouse_down == MOUSE_LEFT {
                let container = &mut self.container_pool.items[container_idx];
                container.bounds.x += self.mouse_delta.x;
                container.bounds.y += self.mouse_delta.y;
            }

            let container_bounds = self.container_pool.items[container_idx].bounds;
            title_bounds = Rectangle::new(
                container_bounds.x, container_bounds.y,
                container_bounds.width, style.title_height,
            );

            if options & NO_CLOSE == 0 {
                let close_id = self.get_identifier("!close".as_bytes());
                let close_size = title_bounds.height;
                let close_rect = Rectangle::new(
                    title_bounds.x + title_bounds.width - close_size,
                    title_bounds.y,
                    close_size,
                    close_size,
                );
                draw_icon(
                    self,
                    1,
                    close_rect,
                    style.colors[ColorIndex::Heading as usize],
                );
                self.update_control(close_id, close_rect, options);
                if self.mouse_pressed == MOUSE_LEFT && close_id == self.focus {
                    let container = &mut self.container_pool.items[container_idx];
                    container.open = false;
                }
            }
        }

        let body = {
            let container = &self.container_pool.items[container_idx];
            let mut b = container.bounds;
            if options & NO_TITLE == 0 {
                b.y += style.title_height;
                b.height -= style.title_height;
            }
            b
        };

        self.push_body(container_idx, body, options);

        if options & NO_RESIZE == 0 {
            let size = style.title_height;
            let resize_id = self.get_identifier("!resize".as_bytes());
            let resize_rect = {
                let cb = self.container_pool.items[container_idx].bounds;
                Rectangle::new(
                    cb.x + cb.width - size,
                    cb.y + cb.height - size,
                    size,
                    size,
                )
            };
            self.update_control(resize_id, resize_rect, options);
            if resize_id == self.focus && self.mouse_down == MOUSE_LEFT {
                let container = &mut self.container_pool.items[container_idx];
                container.bounds.width = cmp::max(96, container.bounds.width + self.mouse_delta.x);
                container.bounds.height = cmp::max(64, container.bounds.height + self.mouse_delta.y);
            }
        }

        if options & AUTO_SIZE != 0 {
            let layout_body = self.current_layout().body;
            let container = &mut self.container_pool.items[container_idx];
            container.bounds.width = container.content.x + (container.bounds.width - layout_body.width);
            container.bounds.height = container.content.y + (container.bounds.height - layout_body.height);
        }

        if options & IS_POPUP != 0
            && self.mouse_pressed != 0
            && self.hover_root != Some(container_idx)
        {
            let container = &mut self.container_pool.items[container_idx];
            container.open = false;
        }

        let container_body = self.container_pool.items[container_idx].body;
        self.push_clip(container_body);
        RESULT_ACTIVE
    }

    pub fn pop_window(&mut self) {
        self.pop_clip();
        self.end_root();
    }

    pub fn open_popup(&mut self, name: &str) {
        let container_idx = self.find_container(name).expect("popup not found");
        self.hover_root = Some(container_idx);
        self.next_root = Some(container_idx);
        let container = &mut self.container_pool.items[container_idx];
        container.bounds = Rectangle::new(self.mouse.x, self.mouse.y, 1, 1);
        container.open = true;
        self.bring_front(container_idx);
    }

    pub fn push_popup(&mut self, name: &str) -> i32 {
        let options = IS_POPUP | AUTO_SIZE | NO_RESIZE | NO_SCROLL | NO_TITLE | IS_CLOSED;
        self.push_window(name, Rectangle::new(0, 0, 0, 0), options)
    }

    pub fn pop_popup(&mut self) {
        self.pop_window();
    }

    pub fn push_panel(&mut self, name: &str, options: i32) {
        self.push_identifier(name.as_bytes());
        let identifier = self.last_identifier;
        let container_idx = self.fetch_container(identifier, options)
            .expect("panel container");
        let bounds = self.next_bounds();
        {
            let container = &mut self.container_pool.items[container_idx];
            container.bounds = bounds;
        }
        if options & NO_FRAME == 0 {
            let draw = self.draw_frame;
            draw(self, bounds, ColorIndex::Panel);
        }
        self.containers_stack.push(container_idx);
        self.push_body(container_idx, bounds, options);
        let body = self.container_pool.items[container_idx].body;
        self.push_clip(body);
    }

    pub fn pop_panel(&mut self) {
        self.pop_clip();
        self.pop_container();
    }

    fn push_body(&mut self, container_idx: usize, mut body: Rectangle, options: i32) {
        let style = unsafe { &*self.style };
        if options & NO_SCROLL == 0 {
            self.render_scrollbars(container_idx, &mut body);
        }
        let scroll = self.container_pool.items[container_idx].scroll;
        self.push_layout(body.expand(-style.padding), scroll);
        self.container_pool.items[container_idx].body = body;
    }

    fn begin_root(&mut self, container_idx: usize) {
        let head_idx = push_jump(self);
        {
            let container = &mut self.container_pool.items[container_idx];
            container.head = head_idx;
            if container.bounds.contains_point(self.mouse)
                && (self.next_root.is_none()
                || container.depth > self.container_pool.items[self.next_root.unwrap()].depth)
            {
                self.next_root = Some(container_idx);
            }
        }
        self.containers_stack.push(container_idx);
        self.roots.push(container_idx);
        self.clips.push(Rectangle::unbounded());
    }

    fn end_root(&mut self) {
        let container_idx = self.containers_stack.last().copied().unwrap();
        let tail_idx = push_jump(self);
        {
            let container = &mut self.container_pool.items[container_idx];
            container.tail = tail_idx;
            self.instructions[container.head] = Instruction::Jump {
                target: container.tail + 1,
            };
        }
        self.clips.pop();
        self.pop_container();
    }

    fn render_scrollbars(&mut self, container_idx: usize, bounds: &mut Rectangle) {
        let style = unsafe { &*self.style };
        let size = style.scroll_size;
        let mut content;
        let body_height;
        let body_width;
        {
            let container = &self.container_pool.items[container_idx];
            content = container.content;
            content.x += style.padding * 2;
            content.y += style.padding * 2;
            body_height = container.body.height;
            body_width = container.body.width;
        }
        self.push_clip(*bounds);
        if content.y > body_height {
            bounds.width -= size;
        }
        if content.x > body_width {
            bounds.height -= size;
        }
        self.render_scrollbar(container_idx, bounds, content, true);
        self.render_scrollbar(container_idx, bounds, content, false);
        self.pop_clip();
    }

    fn render_scrollbar(
        &mut self,
        container_idx: usize,
        bounds: &Rectangle,
        content: Vector,
        vertical: bool,
    ) {
        let (content_size, viewport_size) = if vertical {
            (content.y, bounds.height)
        } else {
            (content.x, bounds.width)
        };

        let max_scroll = content_size - viewport_size;
        if max_scroll <= 0 || viewport_size <= 0 {
            let container = &mut self.container_pool.items[container_idx];
            if vertical {
                container.scroll.y = 0;
            } else {
                container.scroll.x = 0;
            }
            return;
        }

        let style = unsafe { &*self.style };
        let mut base = *bounds;
        let identifier = if vertical {
            base.x = bounds.x + bounds.width;
            base.width = style.scroll_size;
            self.get_identifier("!scrollbar_y".as_bytes())
        } else {
            base.y = bounds.y + bounds.height;
            base.height = style.scroll_size;
            self.get_identifier("!scrollbar_x".as_bytes())
        };

        self.update_control(identifier, base, 0);

        let scroll = if vertical {
            self.container_pool.items[container_idx].scroll.y
        } else {
            self.container_pool.items[container_idx].scroll.x
        };

        let mut thumb = base;
        if vertical {
            thumb.height = cmp::max(
                style.thumb_size,
                base.height * viewport_size / content_size,
            );
            thumb.y += scroll * (base.height - thumb.height) / max_scroll;
        } else {
            thumb.width = cmp::max(
                style.thumb_size,
                base.width * viewport_size / content_size,
            );
            thumb.x += scroll * (base.width - thumb.width) / max_scroll;
        }

        if self.focus == identifier && self.mouse_down == MOUSE_LEFT {
            let container = &mut self.container_pool.items[container_idx];
            if vertical {
                let delta = self.mouse_delta.y;
                container.scroll.y += delta * content_size / base.height;
            } else {
                let delta = self.mouse_delta.x;
                container.scroll.x += delta * content_size / base.width;
            }
        } else if self.hover == identifier && self.mouse_pressed == MOUSE_LEFT {
            let container = &mut self.container_pool.items[container_idx];
            if vertical {
                if self.mouse.y < thumb.y {
                    container.scroll.y -= viewport_size;
                } else if self.mouse.y > thumb.y + thumb.height {
                    container.scroll.y += viewport_size;
                }
            } else {
                if self.mouse.x < thumb.x {
                    container.scroll.x -= viewport_size;
                } else if self.mouse.x > thumb.x + thumb.width {
                    container.scroll.x += viewport_size;
                }
            }
        }

        {
            let container = &mut self.container_pool.items[container_idx];
            if vertical {
                container.scroll.y = container.scroll.y.clamp(0, max_scroll);
                let scroll = container.scroll.y;
                thumb.y = base.y + scroll * (base.height - thumb.height) / max_scroll;
            } else {
                container.scroll.x = container.scroll.x.clamp(0, max_scroll);
                let scroll = container.scroll.x;
                thumb.x = base.x + scroll * (base.width - thumb.width) / max_scroll;
            }
        }

        let draw = self.draw_frame;
        draw(self, base, ColorIndex::Scroll);
        draw(self, thumb, ColorIndex::Thumb);

        if self.mouse_over(*bounds) {
            self.scroll_target = Some(container_idx);
        }
    }
}