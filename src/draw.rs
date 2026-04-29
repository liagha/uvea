use crate::context::Context;
use crate::types::*;

pub fn set_clip(context: &mut Context, bounds: Rectangle) {
    context.instructions.push(Instruction::Clip { bounds });
}

pub fn draw_rectangle(context: &mut Context, bounds: Rectangle, color: Color) {
    let clipped = bounds.intersect(context.current_clip());
    if clipped.width <= 0 || clipped.height <= 0 {
        return;
    }
    context.instructions.push(Instruction::Rectangle {
        bounds: clipped,
        radius: 0.0,
        mode: 0,
        color,
    });
}

pub fn draw_box(context: &mut Context, bounds: Rectangle, color: Color) {
    let left = Rectangle::new(bounds.x, bounds.y, 1, bounds.height);
    let right = Rectangle::new(bounds.x + bounds.width - 1, bounds.y, 1, bounds.height);
    let top = Rectangle::new(bounds.x + 1, bounds.y, bounds.width - 2, 1);
    let bottom = Rectangle::new(bounds.x + 1, bounds.y + bounds.height - 1, bounds.width - 2, 1);
    draw_rectangle(context, left, color);
    draw_rectangle(context, right, color);
    draw_rectangle(context, top, color);
    draw_rectangle(context, bottom, color);
}

pub fn draw_rounded_rectangle(context: &mut Context, bounds: Rectangle, radius: f32, color: Color) {
    if radius <= 0.0 {
        draw_rectangle(context, bounds, color);
        return;
    }
    let clipped = bounds.intersect(context.current_clip());
    if clipped.width <= 0 || clipped.height <= 0 {
        return;
    }
    context.instructions.push(Instruction::Rectangle {
        bounds: clipped,
        radius,
        mode: 1,
        color,
    });
}

pub fn draw_rounded_box(context: &mut Context, bounds: Rectangle, radius: f32, color: Color) {
    if radius <= 0.0 {
        draw_box(context, bounds, color);
        return;
    }
    let clipped = bounds.intersect(context.current_clip());
    if clipped.width <= 0 || clipped.height <= 0 {
        return;
    }
    context.instructions.push(Instruction::Rectangle {
        bounds: clipped,
        radius,
        mode: 3,
        color,
    });
}

pub fn draw_text(
    context: &mut Context,
    font: Font,
    text: &[u8],
    position: Vector,
    color: Color,
) {
    let text_width = context.text_width.unwrap()(font, text);
    let text_height = context.text_height.unwrap()(font);
    let bounds = Rectangle::new(position.x, position.y, text_width, text_height);

    let clipped = context.check_clip(bounds);
    if clipped == CLIP_ALL {
        return;
    }
    if clipped == CLIP_PARTIAL {
        set_clip(context, context.current_clip());
    }

    let offset = context.text_buffer.len();
    context.text_buffer.extend_from_slice(text);
    let length = text.len();

    context.instructions.push(Instruction::Text {
        font,
        position,
        color,
        string_offset: offset,
        string_length: length,
    });

    if clipped != 0 {
        set_clip(context, Rectangle::unbounded());
    }
}

pub fn draw_icon(context: &mut Context, identifier: i32, bounds: Rectangle, color: Color) {
    let clipped = context.check_clip(bounds);
    if clipped == CLIP_ALL {
        return;
    }
    if clipped == CLIP_PARTIAL {
        set_clip(context, context.current_clip());
    }
    context.instructions.push(Instruction::Icon {
        identifier,
        bounds,
        color,
    });
    if clipped != 0 {
        set_clip(context, Rectangle::unbounded());
    }
}

pub fn draw_image(context: &mut Context, source: Image, bounds: Rectangle, tint: Color) {
    let clipped = context.check_clip(bounds);
    if clipped == CLIP_ALL {
        return;
    }
    if clipped == CLIP_PARTIAL {
        set_clip(context, context.current_clip());
    }
    context.instructions.push(Instruction::Image {
        source,
        bounds,
        tint,
    });
    if clipped != 0 {
        set_clip(context, Rectangle::unbounded());
    }
}

pub fn push_jump(context: &mut Context) -> usize {
    let idx = context.instructions.len();
    context.instructions.push(Instruction::Jump { target: 0 });
    idx
}