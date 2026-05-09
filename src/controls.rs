use crate::{
    context::Context,
    types::*,
};

fn mouse_over(ctx: &Context, bounds: Rect) -> bool {
    bounds.contains(ctx.mouse) && ctx.clip().contains(ctx.mouse) && ctx.hovering()
}

pub fn update(ctx: &mut Context, id: Id, bounds: Rect, options: Options) {
    let over = mouse_over(ctx, bounds);

    if ctx.focus == id {
        ctx.focus_updated = true;
    }
    if options.contains(Options::PASSIVE) {
        return;
    }
    if over && ctx.mouse_down == 0 {
        ctx.hover = id;
    }
    if ctx.focus == id {
        if ctx.mouse_pressed != 0 && !over {
            ctx.set_focus(0);
        }
        if ctx.mouse_down == 0 && !options.contains(Options::HOLD_FOCUS) {
            ctx.set_focus(0);
        }
    }
    if ctx.hover == id {
        if ctx.mouse_pressed != 0 {
            ctx.set_focus(id);
        } else if !over {
            ctx.hover = 0;
        }
    }
}

fn frame_color(ctx: &Context, id: Id, base: ColorSlot) -> ColorSlot {
    if ctx.focus == id {
        match base {
            ColorSlot::Button => ColorSlot::ButtonFocus,
            ColorSlot::Input => ColorSlot::InputFocus,
            _ => base,
        }
    } else if ctx.hover == id {
        match base {
            ColorSlot::Button => ColorSlot::ButtonHover,
            ColorSlot::Input => ColorSlot::InputHover,
            _ => base,
        }
    } else {
        base
    }
}

fn draw_frame(ctx: &mut Context, id: Id, bounds: Rect, base: ColorSlot, options: Options) {
    if options.contains(Options::NO_FRAME) {
        return;
    }
    let slot = frame_color(ctx, id, base);
    let color = ctx.style.color(slot);
    let radius = ctx.style.corner_radius;
    if radius > 0.0 {
        ctx.draws.rounded_rect(bounds, color, radius);
        let border = ctx.style.color(ColorSlot::Border);
        if border.a > 0 {
            ctx.draws.rounded_outline(bounds.expand(1), border, radius);
        }
    } else {
        ctx.draws.rect(bounds, color);
        let border = ctx.style.color(ColorSlot::Border);
        if border.a > 0 {
            ctx.draws.border(bounds, border);
        }
    }
}

fn draw_text_aligned(
    ctx: &mut Context,
    text: &str,
    bounds: Rect,
    color: ColorSlot,
    _options: Options,
) {
    let font = ctx.style.font;
    let padding = ctx.style.padding;
    let color = ctx.style.color(color);
    ctx.push_clip(bounds);
    let position = Vec2::new(bounds.x + padding, bounds.y + bounds.h / 4);
    ctx.draws.text(font, text.to_string(), position, color);
    ctx.pop_clip();
}

pub fn button(ctx: &mut Context, label: &str, icon: Option<Icon>, options: Options) -> Response {
    let id = ctx.id_str(label);
    let bounds = ctx.next_bounds();
    update(ctx, id, bounds, options);

    let mut result = Response::empty();
    if ctx.mouse_pressed == Mouse::Left as u32 && ctx.focus == id {
        result |= Response::SUBMIT;
    }

    draw_frame(ctx, id, bounds, ColorSlot::Button, options);
    draw_text_aligned(ctx, label, bounds, ColorSlot::Text, options);
    if let Some(icon) = icon {
        let color = ctx.style.color(ColorSlot::Text);
        ctx.draws.icon(icon, bounds, color);
    }
    result
}

pub fn checkbox(ctx: &mut Context, label: &str, state: &mut bool) -> Response {
    // Use the address of state as identifier bytes
    let addr = (state as *const bool as usize).to_le_bytes();
    let id = ctx.id(&addr);
    let bounds = ctx.next_bounds();
    let box_rect = Rect::new(bounds.x, bounds.y, bounds.h, bounds.h);
    update(ctx, id, bounds, Options::empty());

    let mut result = Response::empty();
    if ctx.mouse_pressed == Mouse::Left as u32 && ctx.focus == id {
        result |= Response::CHANGE;
        *state = !*state;
    }

    draw_frame(ctx, id, box_rect, ColorSlot::Input, Options::empty());
    if *state {
        let color = ctx.style.color(ColorSlot::Text);
        ctx.draws.icon(Icon::Check, box_rect, color);
    }
    let label_bounds = Rect::new(
        bounds.x + box_rect.w,
        bounds.y,
        bounds.w - box_rect.w,
        bounds.h,
    );
    draw_text_aligned(ctx, label, label_bounds, ColorSlot::Text, Options::empty());
    result
}

pub fn label(ctx: &mut Context, text: &str) {
    let bounds = ctx.next_bounds();
    draw_text_aligned(ctx, text, bounds, ColorSlot::Text, Options::empty());
}

pub fn slider(
    ctx: &mut Context,
    value: &mut f32,
    low: f32,
    high: f32,
    step: f32,
    format: &str,
    options: Options,
) -> Response {
    let addr = (value as *const f32 as usize).to_le_bytes();
    let id = ctx.id(&addr);
    let base = ctx.next_bounds();
    let last = *value;
    let mut current = last;
    update(ctx, id, base, options);

    if ctx.focus == id && (ctx.mouse_down | ctx.mouse_pressed) == Mouse::Left as u32 {
        current = low + (ctx.mouse.x - base.x) as f32 * (high - low) / base.w as f32;
        if step > 0.0 {
            current = ((current + step * 0.5) / step).floor() * step;
        }
    }

    *value = current.clamp(low, high);
    let mut result = Response::empty();
    if last != *value {
        result |= Response::CHANGE;
    }

    draw_frame(ctx, id, base, ColorSlot::Input, options);
    let thumb_w = ctx.style.thumb_size;
    let shift = ((*value - low) * (base.w - thumb_w) as f32 / (high - low)) as i32;
    let thumb = Rect::new(base.x + shift, base.y, thumb_w, base.h);
    draw_frame(ctx, id, thumb, ColorSlot::Button, options);

    let label = format!("{:.2}", current);
    draw_text_aligned(ctx, &label, base, ColorSlot::Text, options | Options::ALIGN_CENTER);
    result
}

pub fn textbox_raw(
    ctx: &mut Context,
    buffer: &mut String,
    capacity: usize,
    id: Id,
    bounds: Rect,
    options: Options,
) -> Response {
    let mut result = Response::empty();
    update(ctx, id, bounds, options | Options::HOLD_FOCUS);
    let focused = ctx.focus == id;

    if focused && ctx.text_focus != id {
        ctx.text_focus = id;
        ctx.text_scroll = 0;
    }

    if focused {
        ctx.cursor = ctx.cursor.min(buffer.len());
        ctx.selection = ctx.selection.min(buffer.len());

        let ctrl = ctx.key_down & Key::Control as u32 != 0;
        let shift = ctx.key_down & Key::Shift as u32 != 0;

        if ctrl {
            if ctx.key_pressed & Key::A as u32 != 0 {
                ctx.selection = 0;
                ctx.cursor = buffer.len();
            }
            if ctx.key_pressed & Key::C as u32 != 0 || ctx.key_pressed & Key::X as u32 != 0 {
                let left = ctx.cursor.min(ctx.selection);
                let right = ctx.cursor.max(ctx.selection);
                if left != right {
                    ctx.clipboard = buffer[left..right].to_string();
                }
                if ctx.key_pressed & Key::X as u32 != 0 && left != right {
                    buffer.replace_range(left..right, "");
                    ctx.cursor = left;
                    ctx.selection = left;
                    result |= Response::CHANGE;
                }
            }
            if ctx.key_pressed & Key::V as u32 != 0 {
                let clip = ctx.clipboard.clone();
                let left = ctx.cursor.min(ctx.selection);
                let right = ctx.cursor.max(ctx.selection);
                if left != right {
                    buffer.replace_range(left..right, "");
                    ctx.cursor = left;
                    ctx.selection = left;
                }
                let insert = (capacity - buffer.len()).min(clip.len());
                if insert > 0 {
                    buffer.insert_str(ctx.cursor, &clip[..insert]);
                    ctx.cursor += insert;
                    ctx.selection = ctx.cursor;
                    result |= Response::CHANGE;
                }
            }
        } else {
            if ctx.key_pressed & Key::Left as u32 != 0 && ctx.cursor > 0 {
                ctx.cursor = prev_char(buffer, ctx.cursor);
                if !shift { ctx.selection = ctx.cursor; }
            }
            if ctx.key_pressed & Key::Right as u32 != 0 && ctx.cursor < buffer.len() {
                ctx.cursor = next_char(buffer, ctx.cursor);
                if !shift { ctx.selection = ctx.cursor; }
            }
            if ctx.key_pressed & (Key::Backspace as u32 | Key::Delete as u32) != 0 {
                let left = ctx.cursor.min(ctx.selection);
                let right = ctx.cursor.max(ctx.selection);
                if left != right {
                    buffer.replace_range(left..right, "");
                    ctx.cursor = left;
                    ctx.selection = left;
                    result |= Response::CHANGE;
                } else if ctx.key_pressed & Key::Backspace as u32 != 0 && ctx.cursor > 0 {
                    let prev = prev_char(buffer, ctx.cursor);
                    buffer.remove(prev);
                    ctx.cursor = prev;
                    ctx.selection = prev;
                    result |= Response::CHANGE;
                } else if ctx.key_pressed & Key::Delete as u32 != 0 && ctx.cursor < buffer.len() {
                    let next = next_char(buffer, ctx.cursor);
                    buffer.replace_range(ctx.cursor..next, "");
                    result |= Response::CHANGE;
                }
            }
        }

        if !ctx.input.is_empty() {
            let text = ctx.input.clone();
            let left = ctx.cursor.min(ctx.selection);
            let right = ctx.cursor.max(ctx.selection);
            if left != right {
                buffer.replace_range(left..right, "");
                ctx.cursor = left;
                ctx.selection = left;
            }
            let insert = (capacity - buffer.len()).min(text.len());
            if insert > 0 {
                buffer.insert_str(ctx.cursor, &text[..insert]);
                ctx.cursor += insert;
                ctx.selection = ctx.cursor;
                result |= Response::CHANGE;
            }
        }

        if !options.contains(Options::MULTILINE) && ctx.key_pressed & Key::Return as u32 != 0 {
            ctx.set_focus(0);
            result |= Response::SUBMIT;
        }
    }

    draw_frame(ctx, id, bounds, ColorSlot::Input, options);
    let font = ctx.style.font;
    let color = ctx.style.color(ColorSlot::Text);
    let pos = Vec2::new(bounds.x + ctx.style.padding, bounds.y + ctx.style.padding);
    ctx.push_clip(bounds);
    ctx.draws.text(font, buffer.clone(), pos, color);

    if focused {
        let cursor_x = pos.x + ctx.cursor as i32 * 8;
        ctx.draws.rect(Rect::new(cursor_x, pos.y, 1, ctx.style.size.y + 4), color);
    }
    ctx.pop_clip();
    result
}

pub fn textbox(ctx: &mut Context, buffer: &mut String, capacity: usize, options: Options) -> Response {
    let addr = (buffer.as_ptr() as usize).to_le_bytes();
    let id = ctx.id(&addr);
    let bounds = ctx.next_bounds();
    textbox_raw(ctx, buffer, capacity, id, bounds, options)
}

fn next_char(s: &str, idx: usize) -> usize {
    let mut i = idx + 1;
    while i < s.len() && (s.as_bytes()[i] & 0xC0) == 0x80 {
        i += 1;
    }
    i
}

fn prev_char(s: &str, idx: usize) -> usize {
    if idx == 0 { return 0; }
    let mut i = idx - 1;
    while i > 0 && (s.as_bytes()[i] & 0xC0) == 0x80 {
        i -= 1;
    }
    i
}