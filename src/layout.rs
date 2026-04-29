// src/layout.rs

use crate::context::Context;
use crate::types::*;
use std::cmp;

impl Context {
    pub fn current_layout(&self) -> &Layout {
        self.layouts.last().expect("layout stack empty")
    }

    pub(crate) fn current_layout_mut(&mut self) -> &mut Layout {
        self.layouts.last_mut().expect("layout stack empty")
    }

    pub fn push_layout(&mut self, body: Rectangle, scroll: Vector) {
        let mut layout = Layout::default();
        layout.body = Rectangle::new(body.x - scroll.x, body.y - scroll.y, body.width, body.height);
        layout.maximum_size = Vector::new(-0x1000000, -0x1000000);
        self.layouts.push(layout);
        self.layout_row(1, None, 0);
    }

    pub fn pop_layout(&mut self) {
        self.layouts.pop().expect("layout stack underflow");
    }

    pub fn begin_column(&mut self) {
        let bounds = self.next_bounds();
        self.push_layout(bounds, Vector::new(0, 0));
    }

    pub fn end_column(&mut self) {
        let b = *self.current_layout();
        self.pop_layout();
        let a = self.current_layout_mut();

        a.position.x = cmp::max(a.position.x, b.position.x + b.body.x - a.body.x);
        a.next_row = cmp::max(a.next_row, b.next_row + b.body.y - a.body.y);
        a.maximum_size.x = cmp::max(a.maximum_size.x, b.maximum_size.x);
        a.maximum_size.y = cmp::max(a.maximum_size.y, b.maximum_size.y);
    }

    pub fn layout_row(&mut self, items: i32, widths: Option<&[i32]>, height: i32) {
        let layout = self.current_layout_mut();
        if let Some(slice) = widths {
            assert!(items as usize <= MAX_WIDTHS);
            let len = slice.len().min(MAX_WIDTHS);
            layout.widths[..len].copy_from_slice(&slice[..len]);
        }
        layout.items = items;
        layout.position = Vector::new(layout.indent, layout.next_row);
        layout.size.y = height;
        layout.item_index = 0;
    }

    pub fn layout_width(&mut self, width: i32) {
        self.current_layout_mut().size.x = width;
    }

    pub fn layout_height(&mut self, height: i32) {
        self.current_layout_mut().size.y = height;
    }

    pub fn layout_next(&mut self, bounds: Rectangle, relative: i32) {
        let layout = self.current_layout_mut();
        layout.next = bounds;
        layout.next_type = relative;
    }

    pub fn next_bounds(&mut self) -> Rectangle {
        let style = unsafe { &*self.style };
        let mut result = Rectangle::default();

        let next_type = {
            let layout = self.current_layout_mut();
            layout.next_type
        };

        if next_type != 0 {
            let layout = self.current_layout_mut();
            let kind = layout.next_type;
            layout.next_type = 0;
            result = layout.next;
            if kind == 2 {
                self.last_bounds = result;
                return result;
            }
        } else {
            let need_new_row;
            let items;
            let height;
            {
                let layout = self.current_layout_mut();
                need_new_row = layout.item_index == layout.items;
                items = layout.items;
                height = layout.size.y;
            }
            if need_new_row {
                self.layout_row(items, None, height);
            }
            {
                let layout = self.current_layout_mut();
                result.x = layout.position.x;
                result.y = layout.position.y;
                result.width = if layout.items > 0 {
                    layout.widths[layout.item_index as usize]
                } else {
                    layout.size.x
                };
                result.height = layout.size.y;

                if result.width == 0 {
                    result.width = style.size.x + style.padding * 2;
                }
                if result.height == 0 {
                    result.height = style.size.y + style.padding * 2;
                }
                if result.width < 0 {
                    result.width += layout.body.width - result.x + 1;
                }
                if result.height < 0 {
                    result.height += layout.body.height - result.y + 1;
                }
                layout.item_index += 1;
            }
        }

        let layout = self.current_layout_mut();
        layout.position.x += result.width + style.spacing;
        layout.next_row = cmp::max(layout.next_row, result.y + result.height + style.spacing);
        result.x += layout.body.x;
        result.y += layout.body.y;
        layout.maximum_size.x = cmp::max(layout.maximum_size.x, result.x + result.width);
        layout.maximum_size.y = cmp::max(layout.maximum_size.y, result.y + result.height);

        self.last_bounds = result;
        result
    }

    pub fn pop_container(&mut self) {
        let container_idx = self.current_container_index();
        let layout = self.current_layout();
        let content_x = layout.maximum_size.x - layout.body.x;
        let content_y = layout.maximum_size.y - layout.body.y;
        {
            let container = &mut self.container_pool.items[container_idx];
            container.content.x = content_x;
            container.content.y = content_y;
        }
        self.containers_stack.pop().expect("container stack empty");
        self.pop_layout();
        self.pop_identifier();
    }
}