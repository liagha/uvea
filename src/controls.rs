// src/controls.rs

use crate::context::Context;
use crate::draw::*;
use crate::types::*;

impl Context {
    fn hovering(&self) -> bool {
        let mut idx = self.containers_stack.len() as isize;
        while idx > 0 {
            idx -= 1;
            let container_idx = self.containers_stack[idx as usize];
            if Some(container_idx) == self.hover_root {
                return true;
            }
            if self.container_pool.items[container_idx].head != 0 {
                break;
            }
        }
        false
    }

    pub fn control_frame(
        &mut self,
        identifier: Identifier,
        bounds: Rectangle,
        color_index: ColorIndex,
        options: i32,
    ) {
        if options & NO_FRAME != 0 {
            return;
        }
        let shift = if self.focus == identifier {
            2
        } else if self.hover == identifier {
            1
        } else {
            0
        };
        let final_index = match color_index {
            ColorIndex::Button => ColorIndex::Button as i32 + shift,
            ColorIndex::Input => ColorIndex::Input as i32 + shift,
            _ => color_index as i32,
        };
        let draw = self.draw_frame;
        draw(self, bounds, unsafe { std::mem::transmute(final_index as u8) });
    }

    pub fn control_text(
        &mut self,
        text: &str,
        bounds: Rectangle,
        color_index: ColorIndex,
        options: i32,
    ) {
        let style = unsafe { &*self.style };
        let font = style.font;
        let width = self.text_width.unwrap()(font, text.as_bytes());
        self.push_clip(bounds);

        let position_y = bounds.y + (bounds.height - self.text_height.unwrap()(font)) / 2;
        let position_x = if options & ALIGN_CENTER != 0 {
            bounds.x + (bounds.width - width) / 2
        } else if options & ALIGN_RIGHT != 0 {
            bounds.x + bounds.width - width - style.padding
        } else {
            bounds.x + style.padding
        };
        draw_text(
            self,
            font,
            text.as_bytes(),
            Vector::new(position_x, position_y),
            style.colors[color_index as usize],
        );
        self.pop_clip();
    }

    pub fn mouse_over(&mut self, bounds: Rectangle) -> bool {
        bounds.contains_point(self.mouse)
            && self.current_clip().contains_point(self.mouse)
            && self.hovering()
    }

    pub fn update_control(
        &mut self,
        identifier: Identifier,
        bounds: Rectangle,
        options: i32,
    ) {
        let over = self.mouse_over(bounds);
        if self.focus == identifier {
            self.focus_updated = true;
        }
        if options & PASSIVE != 0 {
            return;
        }
        if over && self.mouse_down == 0 {
            self.hover = identifier;
        }
        if self.focus == identifier {
            if self.mouse_pressed != 0 && !over {
                self.set_focus(0);
            }
            if self.mouse_down == 0 && options & HOLD_FOCUS == 0 {
                self.set_focus(0);
            }
        }
        if self.hover == identifier {
            if self.mouse_pressed != 0 {
                self.set_focus(identifier);
            } else if !over {
                self.hover = 0;
            }
        }
    }

    pub fn build_text(&mut self, text: &str) {
        let font = unsafe { &*self.style }.font;
        let color = unsafe { &*self.style }.colors[ColorIndex::Text as usize];
        let height = self.text_height.unwrap()(font);
        let width = -1;

        self.begin_column();
        self.layout_row(1, Some(&[width]), height);

        let mut pointer = 0usize;
        while pointer < text.len() {
            let bounds = self.next_bounds();
            let line_start = pointer;
            let mut step = 0;
            let mut line_end = pointer;
            loop {
                let word_start = pointer;
                let word_end = find_word_boundary(text, pointer);
                let token = &text.as_bytes()[word_start..word_end];
                let token_width = self.text_width.unwrap()(font, token);

                if step + token_width > bounds.width && line_end != word_start {
                    break;
                }
                step += token_width;
                if word_end < text.len() && text.as_bytes()[word_end] == b' ' {
                    step += self.text_width.unwrap()(font, &[text.as_bytes()[word_end]]);
                }
                line_end = word_end;
                pointer = if word_end < text.len() { word_end + 1 } else { word_end };
                if line_end >= text.len() || text.as_bytes()[line_end] == b'\n' {
                    break;
                }
            }

            draw_text(
                self,
                font,
                &text.as_bytes()[line_start..line_end],
                Vector::new(bounds.x, bounds.y),
                color,
            );

            pointer = if line_end < text.len() { line_end + 1 } else { line_end };
        }
        self.end_column();
    }

    pub fn build_label(&mut self, text: &str) {
        let bounds = self.next_bounds();
        self.control_text(text, bounds, ColorIndex::Text, 0);
    }

    pub fn build_image(&mut self, source: Image) {
        let bounds = self.next_bounds();
        draw_image(self, source, bounds, Color::new(255, 255, 255, 255));
    }

    pub fn draw_button(&mut self, label: Option<&str>, icon: i32, options: i32) -> i32 {
        let mut result = 0;
        let identifier = if let Some(s) = label {
            self.get_identifier(s.as_bytes())
        } else {
            self.get_identifier(&icon.to_ne_bytes())
        };
        let bounds = self.next_bounds();
        self.update_control(identifier, bounds, options);

        if self.mouse_pressed == MOUSE_LEFT && self.focus == identifier {
            result |= RESULT_SUBMIT;
        }
        self.control_frame(identifier, bounds, ColorIndex::Button, options);
        if let Some(s) = label {
            self.control_text(s, bounds, ColorIndex::Text, options);
        }
        if icon != 0 {
            draw_icon(
                self,
                icon,
                bounds,
                unsafe { &*self.style }.colors[ColorIndex::Text as usize],
            );
        }
        result
    }

    pub fn draw_checkbox(&mut self, label: &str, state: &mut bool) -> i32 {
        let mut result = 0;
        let identifier = self.get_identifier(&(state as *const _ as usize).to_ne_bytes());
        let bounds = self.next_bounds();
        let box_rect = Rectangle::new(bounds.x, bounds.y, bounds.height, bounds.height);
        self.update_control(identifier, bounds, 0);

        if self.mouse_pressed == MOUSE_LEFT && self.focus == identifier {
            result |= RESULT_CHANGE;
            *state = !*state;
        }
        self.control_frame(identifier, box_rect, ColorIndex::Input, 0);
        if *state {
            draw_icon(
                self,
                2,
                box_rect,
                unsafe { &*self.style }.colors[ColorIndex::Text as usize],
            );
        }

        let label_bounds = Rectangle::new(
            bounds.x + box_rect.width,
            bounds.y,
            bounds.width - box_rect.width,
            bounds.height,
        );
        self.control_text(label, label_bounds, ColorIndex::Text, 0);
        result
    }

    pub fn raw_textbox(
        &mut self,
        buffer: &mut [u8],
        identifier: Identifier,
        bounds: Rectangle,
        options: i32,
    ) -> i32 {
        let mut result = 0;
        let mut length = strlen(buffer);
        self.update_control(identifier, bounds, options | HOLD_FOCUS);
        let focused = self.focus == identifier;
        let mut scroll_x = if focused && self.text_focus == identifier {
            self.text_scroll
        } else {
            0
        };
        if focused && self.text_focus != identifier {
            self.text_focus = identifier;
            self.text_scroll = 0;
            scroll_x = 0;
        }

        let style = unsafe { &*self.style };
        let font = style.font;
        let height = self.text_height.unwrap()(font);
        let mut shift_x = bounds.x + style.padding - self.scroll.x - scroll_x;
        let shift_y = bounds.y
            + if options & MULTILINE != 0 {
            style.padding - self.scroll.y
        } else {
            (bounds.height - height) / 2
        };

        if focused {
            self.cursor = utf8_clamp_boundary(buffer, self.cursor);
            self.selection = utf8_clamp_boundary(buffer, self.selection);
            self.cursor = self.cursor.min(length);
            self.selection = self.selection.min(length);

            if self.mouse_down == MOUSE_LEFT && self.mouse_over(bounds) {
                let target_x = self.mouse.x - shift_x;
                let target_y = self.mouse.y - shift_y;
                let pos = map_cursor(self, buffer, target_x, target_y, height);
                if self.mouse_pressed != 0 {
                    self.selection = pos;
                }
                self.cursor = pos;
            }

            if self.key_down & KEY_CONTROL != 0 {
                if self.key_pressed & KEY_A != 0 {
                    self.selection = 0;
                    self.cursor = length;
                }
                if self.key_pressed & (KEY_C | KEY_X) != 0 {
                    let left = self.cursor.min(self.selection);
                    let right = self.cursor.max(self.selection);
                    if left != right {
                        if let Some(clip_fn) = self.set_clipboard {
                            let clip = &buffer[left..right];
                            let text = String::from_utf8_lossy(clip);
                            clip_fn(self, &text);
                        }
                    }
                    if self.key_pressed & KEY_X != 0 && left != right {
                        let delta = right - left;
                        buffer.copy_within(right.., left);
                        length -= delta;
                        self.cursor = left;
                        self.selection = left;
                        result |= RESULT_CHANGE;
                    }
                }
                if self.key_pressed & KEY_V != 0 {
                    if let Some(clip_fn) = self.get_clipboard {
                        let clip = clip_fn(self);
                        let clip_bytes = clip.as_bytes();
                        let left = self.cursor.min(self.selection);
                        let right = self.cursor.max(self.selection);
                        if left != right {
                            buffer.copy_within(right.., left);
                            length -= right - left;
                            self.cursor = left;
                            self.selection = left;
                        }
                        let insert = clip_bytes.len().min(buffer.len() - length - 1);
                        if insert > 0 {
                            let tail = length - self.cursor;
                            buffer.copy_within(self.cursor..self.cursor + tail, self.cursor + insert);
                            buffer[self.cursor..self.cursor + insert].copy_from_slice(&clip_bytes[..insert]);
                            self.cursor += insert;
                            self.selection = self.cursor;
                            length += insert;
                            result |= RESULT_CHANGE;
                        }
                    }
                }
            } else {
                if self.key_pressed & KEY_LEFT != 0 {
                    if self.cursor > 0 {
                        self.cursor = utf8_prev_index(buffer, self.cursor);
                    }
                    if self.key_down & KEY_SHIFT == 0 {
                        self.selection = self.cursor;
                    }
                }
                if self.key_pressed & KEY_RIGHT != 0 {
                    if self.cursor < length {
                        self.cursor = utf8_next_index(buffer, self.cursor);
                    }
                    if self.key_down & KEY_SHIFT == 0 {
                        self.selection = self.cursor;
                    }
                }
                if options & MULTILINE != 0 {
                    if self.key_pressed & KEY_UP != 0 {
                        let pos = get_cursor_position(self, buffer, self.cursor);
                        self.cursor = map_cursor(self, buffer, pos.x, pos.y - height, height);
                        if self.key_down & KEY_SHIFT == 0 {
                            self.selection = self.cursor;
                        }
                    }
                    if self.key_pressed & KEY_DOWN != 0 {
                        let pos = get_cursor_position(self, buffer, self.cursor);
                        self.cursor = map_cursor(self, buffer, pos.x, pos.y + height, height);
                        if self.key_down & KEY_SHIFT == 0 {
                            self.selection = self.cursor;
                        }
                    }
                }
            }

            let left = self.cursor.min(self.selection);
            let right = self.cursor.max(self.selection);

            if self.key_down & KEY_CONTROL == 0
                && (self.key_pressed & (KEY_BACKSPACE | KEY_DELETE) != 0)
                && length > 0
            {
                if left != right {
                    buffer.copy_within(right.., left);
                    length -= right - left;
                    self.cursor = left;
                    self.selection = left;
                    result |= RESULT_CHANGE;
                } else if self.key_pressed & KEY_BACKSPACE != 0 && self.cursor > 0 {
                    let prev = utf8_prev_index(buffer, self.cursor);
                    let delta = self.cursor - prev;
                    buffer.copy_within(self.cursor.., prev);
                    self.cursor = prev;
                    self.selection = prev;
                    length -= delta;
                    result |= RESULT_CHANGE;
                } else if self.key_pressed & KEY_DELETE != 0 && self.cursor < length {
                    let next = utf8_next_index(buffer, self.cursor);
                    let delta = next - self.cursor;
                    buffer.copy_within(next.., self.cursor);
                    length -= delta;
                    result |= RESULT_CHANGE;
                }
            }

            let mut input_len = self.input.len();
            if input_len > 0 && left != right {
                buffer.copy_within(right.., left);
                self.cursor = left;
                self.selection = left;
                length -= right - left;
                input_len = self.input.len();
            }

            if options & MULTILINE != 0 && self.key_pressed & KEY_RETURN != 0 {
                self.input.clear();
                self.input.push('\n');
                input_len = 1;
            }

            let insert = input_len.min(buffer.len() - length - 1);
            if insert > 0 {
                let tail = length - self.cursor;
                buffer.copy_within(self.cursor..self.cursor + tail, self.cursor + insert);
                let input_bytes = self.input.as_bytes();
                buffer[self.cursor..self.cursor + insert].copy_from_slice(&input_bytes[..insert]);
                self.cursor += insert;
                self.selection = self.cursor;
                length += insert;
                result |= RESULT_CHANGE;
            }

            if options & MULTILINE == 0 && self.key_pressed & KEY_RETURN != 0 {
                self.set_focus(0);
                result |= RESULT_SUBMIT;
            }

            let cursor_x = get_cursor_position(self, buffer, self.cursor).x;
            let (line_start, line_end) = get_text_line(buffer, self.cursor);
            let line_width =
                self.text_width.unwrap()(font, &buffer[line_start..line_end]);
            let view_width = bounds.width - style.padding * 2;
            let max_scroll = line_width - view_width;
            if view_width > 0 && max_scroll > 0 {
                if cursor_x < scroll_x {
                    scroll_x = cursor_x;
                } else if cursor_x > scroll_x + view_width - 1 {
                    scroll_x = cursor_x - view_width + 1;
                }
                self.text_scroll = scroll_x.clamp(0, max_scroll);
            } else {
                self.text_scroll = 0;
            }
        }

        shift_x = bounds.x + style.padding - self.scroll.x
            - if focused && self.text_focus == identifier { self.text_scroll } else { 0 };

        self.control_frame(identifier, bounds, ColorIndex::Input, options);

        let text_color = style.colors[ColorIndex::Text as usize];
        let select_color = style.colors[ColorIndex::Selection as usize];

        self.push_clip(bounds);

        if self.focus == identifier {
            let left = self.cursor.min(self.selection);
            let right = self.cursor.max(self.selection);

            if left != right {
                let mut start = left;
                while start < right {
                    let mut end = start;
                    while end < right && buffer[end] != b'\n' {
                        end = utf8_next_index(buffer, end);
                    }
                    let pos = get_cursor_position(self, buffer, start);
                    let sel_width = self.text_width.unwrap()(font, &buffer[start..end]);
                    draw_rectangle(
                        self,
                        Rectangle::new(shift_x + pos.x, shift_y + pos.y, sel_width, height),
                        select_color,
                    );
                    if end < right && buffer.get(end) == Some(&b'\n') {
                        start = end + 1;
                    } else {
                        start = end;
                    }
                }
            }

            let cur_pos = get_cursor_position(self, buffer, self.cursor);
            draw_rectangle(
                self,
                Rectangle::new(shift_x + cur_pos.x, shift_y + cur_pos.y, 1, height),
                text_color,
            );
        }

        let mut i = 0;
        let mut current_y = 0;
        while i <= length {
            let line_start = i;
            while i < length && buffer[i] != b'\n' {
                i += 1;
            }
            draw_text(
                self,
                font,
                &buffer[line_start..i],
                Vector::new(shift_x, shift_y + current_y),
                text_color,
            );
            current_y += height;
            if i == length {
                break;
            }
            i += 1;
        }

        self.pop_clip();
        result
    }

    pub fn draw_textbox(
        &mut self,
        buffer: &mut [u8],
        options: i32,
    ) -> i32 {
        let identifier = self.get_identifier(&(buffer.as_ptr() as usize).to_ne_bytes());
        let bounds = self.next_bounds();
        self.raw_textbox(buffer, identifier, bounds, options)
    }

    pub fn draw_slider(
        &mut self,
        value: &mut Real,
        low: Real,
        high: Real,
        step: Real,
        format: &str,
        options: i32,
    ) -> i32 {
        let mut result = 0;
        let identifier = self.get_identifier(&(value as *const _ as usize).to_ne_bytes());
        let base = self.next_bounds();
        let last = *value;
        let mut current = last;

        if number_textbox(self, &mut current, base, identifier) {
            return result;
        }
        self.update_control(identifier, base, options);

        if self.focus == identifier && self.mouse_down | self.mouse_pressed == MOUSE_LEFT {
            current = low + (self.mouse.x - base.x) as f32 * (high - low) / base.width as f32;
            if step > 0.0 {
                current = ((current + step / 2.0) / step).trunc() * step;
            }
        }

        current = current.clamp(low, high);
        *value = current;
        if last != current {
            result |= RESULT_CHANGE;
        }

        self.control_frame(identifier, base, ColorIndex::Input, options);

        let thumb_width = unsafe { &*self.style }.thumb_size;
        let shift = (current - low) * (base.width - thumb_width) as f32 / (high - low) as f32;
        let thumb = Rectangle::new(base.x + shift as i32, base.y, thumb_width, base.height);
        self.control_frame(identifier, thumb, ColorIndex::Button, options);

        let mut buf = [0u8; MAX_FORMAT + 1];
        let text = if let Ok(()) = format_args_buf(&mut buf, format, current) {
            std::str::from_utf8(&buf).unwrap_or("?")
        } else {
            "?"
        };
        self.control_text(text, base, ColorIndex::Text, options);

        result
    }

    pub fn draw_number(
        &mut self,
        value: &mut Real,
        step: Real,
        format: &str,
        options: i32,
    ) -> i32 {
        let mut result = 0;
        let identifier = self.get_identifier(&(value as *const _ as usize).to_ne_bytes());
        let base = self.next_bounds();
        let last = *value;

        if number_textbox(self, value, base, identifier) {
            return result;
        }
        self.update_control(identifier, base, options);

        if self.focus == identifier && self.mouse_down == MOUSE_LEFT {
            *value += self.mouse_delta.x as Real * step;
        }
        if *value != last {
            result |= RESULT_CHANGE;
        }

        self.control_frame(identifier, base, ColorIndex::Input, options);

        let mut buf = [0u8; MAX_FORMAT + 1];
        let text = if let Ok(()) = format_args_buf(&mut buf, format, *value) {
            std::str::from_utf8(&buf).unwrap_or("?")
        } else {
            "?"
        };
        self.control_text(text, base, ColorIndex::Text, options);

        result
    }
}

fn number_textbox(
    context: &mut Context,
    value: &mut Real,
    bounds: Rectangle,
    identifier: Identifier,
) -> bool {
    if context.mouse_pressed == MOUSE_LEFT
        && context.key_down & KEY_SHIFT != 0
        && context.hover == identifier
    {
        context.number_edit = identifier;
        let buf = format!("{:.3}", *value);
        context.number_buffer[..buf.len()].copy_from_slice(buf.as_bytes());
    }
    if context.number_edit == identifier {
        let mut num_buf = context.number_buffer;
        let result = context.raw_textbox(&mut num_buf, identifier, bounds, 0);
        if result & RESULT_SUBMIT != 0 || context.focus != identifier {
            if let Ok(num) = String::from_utf8_lossy(&num_buf).trim().parse() {
                *value = num;
            }
            context.number_edit = 0;
            return false;
        }
        return true;
    }
    false
}

fn utf8_next_index(data: &[u8], index: usize) -> usize {
    if index >= data.len() {
        return data.len();
    }
    let byte = data[index];
    if byte & 0x80 == 0 {
        return index + 1;
    }
    if byte & 0xE0 == 0xC0 {
        return (index + 2).min(data.len());
    }
    if byte & 0xF0 == 0xE0 {
        return (index + 3).min(data.len());
    }
    if byte & 0xF8 == 0xF0 {
        return (index + 4).min(data.len());
    }
    index + 1
}

fn utf8_prev_index(data: &[u8], index: usize) -> usize {
    if index == 0 {
        return 0;
    }
    let mut i = index - 1;
    while i > 0 && data[i] & 0xC0 == 0x80 {
        i -= 1;
    }
    i
}

fn utf8_clamp_boundary(data: &[u8], index: usize) -> usize {
    if index >= data.len() {
        return data.len();
    }
    let mut i = index;
    while i > 0 && data[i] & 0xC0 == 0x80 {
        i -= 1;
    }
    i
}

fn get_cursor_position(context: &Context, buffer: &[u8], index: usize) -> Vector {
    let font = unsafe { &*context.style }.font;
    let height = context.text_height.unwrap()(font);
    let mut current_y = 0;
    let mut line_start = 0;
    let mut i = 0;
    while i < index && i < buffer.len() {
        if buffer[i] == b'\n' {
            current_y += height;
            line_start = i + 1;
        }
        i = utf8_next_index(buffer, i);
    }
    let current_x = context.text_width.unwrap()(font, &buffer[line_start..index]);
    Vector::new(current_x, current_y)
}

fn map_cursor(
    context: &Context,
    buffer: &[u8],
    target_x: i32,
    target_y: i32,
    height: i32,
) -> usize {
    let font = unsafe { &*context.style }.font;
    let mut index = 0;
    let mut current_y = 0;
    while index < buffer.len() {
        if target_y < current_y + height {
            break;
        }
        if buffer[index] == b'\n' {
            current_y += height;
            index += 1;
        } else {
            index = utf8_next_index(buffer, index);
        }
    }
    let line_start = index;
    while index < buffer.len() && buffer[index] != b'\n' {
        let next = utf8_next_index(buffer, index);
        let width = context.text_width.unwrap()(font, &buffer[line_start..next]);
        let prev_width = context.text_width.unwrap()(font, &buffer[line_start..index]);
        if target_x < prev_width + (width - prev_width) / 2 {
            break;
        }
        index = next;
    }
    index
}

fn get_text_line(buffer: &[u8], index: usize) -> (usize, usize) {
    let cursor = index.max(0);
    let mut marker = 0;
    while marker < buffer.len() && marker < cursor {
        if buffer[marker] == b'\n' {
            marker += 1;
            break;
        }
        marker = utf8_next_index(buffer, marker);
    }
    while marker > 0 && buffer[marker - 1] != b'\n' {
        marker = utf8_prev_index(buffer, marker);
    }
    let start = marker;
    let mut end = marker;
    while end < buffer.len() && buffer[end] != b'\n' {
        end = utf8_next_index(buffer, end);
    }
    (start, end)
}

fn strlen(buffer: &[u8]) -> usize {
    buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len())
}

fn find_word_boundary(text: &str, from: usize) -> usize {
    let bytes = text.as_bytes();
    let mut end = from;
    while end < bytes.len() && bytes[end] != b' ' && bytes[end] != b'\n' {
        end = utf8_next_index(bytes, end);
    }
    end
}

fn format_args_buf(buf: &mut [u8], fmt: &str, value: Real) -> Result<(), ()> {
    let fmt_bytes = fmt.as_bytes();
    if fmt_bytes.len() < 2 || fmt_bytes[0] != b'%' {
        return Err(());
    }
    let mut i = 1;
    let mut width = 0usize;
    let mut precision = 6usize;
    while i < fmt_bytes.len() && fmt_bytes[i].is_ascii_digit() {
        width = width * 10 + (fmt_bytes[i] - b'0') as usize;
        i += 1;
    }
    if i < fmt_bytes.len() && fmt_bytes[i] == b'.' {
        i += 1;
        precision = 0;
        while i < fmt_bytes.len() && fmt_bytes[i].is_ascii_digit() {
            precision = precision * 10 + (fmt_bytes[i] - b'0') as usize;
            i += 1;
        }
    }
    if i >= fmt_bytes.len() || fmt_bytes[i] != b'f' {
        return Err(());
    }

    let formatted = format!("{:width$.precision$}", value, width = if width > 0 { width } else { 1 }, precision = precision);
    let bytes = formatted.as_bytes();
    let len = bytes.len().min(buf.len() - 1);
    buf[..len].copy_from_slice(&bytes[..len]);
    buf[len] = 0;
    Ok(())
}