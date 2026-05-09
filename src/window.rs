use crate::{
    context::Context,
    controls,
    types::*,
};

pub fn begin_window(
    ctx: &mut Context,
    title: &str,
    initial: Rect,
    options: Options,
) -> Response {
    let id = ctx.id_str(title);
    let closed = options.contains(Options::IS_CLOSED);
    let Some(idx) = ctx.fetch_container(id, closed) else {
        return Response::empty();
    };

    {
        let container = &mut ctx.context_containers()[idx];
        if !container.open {
            return Response::empty();
        }
        if container.bounds.w == 0 {
            container.bounds = initial;
        }
    }

    ctx.push_id(id.to_le_bytes().as_ref());
    ctx.push_container(idx);

    let bounds = ctx.context_containers()[idx].bounds;
    let mut body = bounds;

    if !options.contains(Options::NO_FRAME) {
        let color = ctx.style.color(ColorSlot::Window);
        ctx.draws.rect(bounds, color);
    }

    if !options.contains(Options::NO_TITLE) {
        let title_h = ctx.style.title_height;
        let title_rect = Rect::new(bounds.x, bounds.y, bounds.w, title_h);
        let title_color = ctx.style.color(ColorSlot::Title);
        ctx.draws.rect(title_rect, title_color);

        let title_id = ctx.id_str("!title");
        controls::update(ctx, title_id, title_rect, options);
        let heading_color = ctx.style.color(ColorSlot::Heading);
        let font = ctx.style.font;
        let pos = Vec2::new(title_rect.x + ctx.style.padding, title_rect.y + title_h / 4);
        ctx.draws.text(font, title.to_string(), pos, heading_color);

        if ctx.focus == title_id && ctx.mouse_down == Mouse::Left as u32 {
            ctx.context_containers()[idx].bounds.x += ctx.delta.x;
            ctx.context_containers()[idx].bounds.y += ctx.delta.y;
        }

        body.y += title_h;
        body.h -= title_h;

        if !options.contains(Options::NO_CLOSE) {
            let close_id = ctx.id_str("!close");
            let close = Rect::new(title_rect.x + title_rect.w - title_h, title_rect.y, title_h, title_h);
            ctx.draws.icon(Icon::Close, close, heading_color);
            controls::update(ctx, close_id, close, options);
            if ctx.mouse_pressed == Mouse::Left as u32 && ctx.focus == close_id {
                ctx.context_containers()[idx].open = false;
            }
        }
    }

    push_body(ctx, idx, body, options);

    if !options.contains(Options::NO_RESIZE) {
        let size = ctx.style.title_height;
        let resize_id = ctx.id_str("!resize");
        let resize = Rect::new(bounds.x + bounds.w - size, bounds.y + bounds.h - size, size, size);
        controls::update(ctx, resize_id, resize, options);
        if ctx.focus == resize_id && ctx.mouse_down == Mouse::Left as u32 {
            let (dx, dy) = (ctx.delta.x, ctx.delta.y);
            ctx.context_containers()[idx].bounds.w = (ctx.context_containers()[idx].bounds.w + dx).max(96);
            ctx.context_containers()[idx].bounds.h = (ctx.context_containers()[idx].bounds.h + dy).max(64);
        }
    }

    // Fix E0499: extract body rect before calling push_clip to avoid double borrow
    let body_rect = ctx.context_containers()[idx].body;
    ctx.push_clip(body_rect);
    Response::ACTIVE
}

pub fn end_window(ctx: &mut Context) {
    ctx.pop_clip();
    ctx.pop_container();
}

pub fn begin_panel(ctx: &mut Context, name: &str, options: Options) {
    ctx.push_id(name.as_bytes());
    let id = ctx.last_id;
    let Some(idx) = ctx.fetch_container(id, false) else { return; };
    let bounds = ctx.next_bounds();
    ctx.context_containers()[idx].bounds = bounds;

    if !options.contains(Options::NO_FRAME) {
        let color = ctx.style.color(ColorSlot::Panel);
        ctx.draws.rect(bounds, color);
    }

    ctx.push_container_raw(idx);
    push_body(ctx, idx, bounds, options);

    // Fix E0499: extract body before calling push_clip
    let body_rect = ctx.context_containers()[idx].body;
    ctx.push_clip(body_rect);
}

pub fn end_panel(ctx: &mut Context) {
    ctx.pop_clip();
    ctx.pop_container();
}

pub fn open_popup(ctx: &mut Context, name: &str) {
    let id = ctx.id_str(name);
    let Some(idx) = ctx.fetch_container(id, false) else { return; };
    ctx.hover_root = Some(idx);
    ctx.next_root = Some(idx);
    ctx.context_containers()[idx].bounds = Rect::new(ctx.mouse.x, ctx.mouse.y, 1, 1);
    ctx.context_containers()[idx].open = true;
    ctx.bring_front(idx);
}

pub fn begin_popup(ctx: &mut Context, name: &str) -> Response {
    let options = Options::IS_POPUP
        | Options::AUTO_SIZE
        | Options::NO_RESIZE
        | Options::NO_SCROLL
        | Options::NO_TITLE
        | Options::IS_CLOSED;
    begin_window(ctx, name, Rect::default(), options)
}

pub fn end_popup(ctx: &mut Context) {
    end_window(ctx);
}

pub fn begin_tree(ctx: &mut Context, label: &str, options: Options) -> Response {
    let id = ctx.id_str(label);
    let active = ctx.fetch_tree(id);
    let expanded = if options.contains(Options::IS_EXPANDED) { !active } else { active };

    let width = -1i32;
    let layout = ctx.layout();
    layout.row(1, &[width], 0);
    let bounds = ctx.next_bounds();
    controls::update(ctx, id, bounds, Options::empty());

    let toggled = ctx.mouse_pressed == Mouse::Left as u32 && ctx.focus == id;
    let new_active = active ^ toggled;

    if active {
        if new_active {
            ctx.update_tree(id);
        } else {
            ctx.deactivate_tree(id);
        }
    } else if new_active {
        ctx.activate_tree(id);
    }

    let hover_color = ctx.style.color(ColorSlot::ButtonHover);
    if ctx.hover == id {
        ctx.draws.rect(bounds, hover_color);
    }

    let icon = if expanded { Icon::Expanded } else { Icon::Collapsed };
    let icon_color = ctx.style.color(ColorSlot::Text);
    let icon_rect = Rect::new(bounds.x, bounds.y, bounds.h, bounds.h);
    ctx.draws.icon(icon, icon_rect, icon_color);

    let text_bounds = Rect::new(
        bounds.x + bounds.h - ctx.style.padding,
        bounds.y,
        bounds.w - bounds.h + ctx.style.padding,
        bounds.h,
    );
    let text_color = ctx.style.color(ColorSlot::Text);
    let font = ctx.style.font;
    ctx.draws.text(font, label.to_string(), Vec2::new(text_bounds.x, text_bounds.y), text_color);

    if expanded {
        ctx.layout().indent += ctx.style.indent;
        ctx.push_id(id.to_le_bytes().as_ref());
        Response::ACTIVE
    } else {
        Response::empty()
    }
}

pub fn end_tree(ctx: &mut Context) {
    ctx.layout().indent -= ctx.style.indent;
    ctx.pop_id();
}

fn push_body(ctx: &mut Context, idx: usize, mut body: Rect, options: Options) {
    if !options.contains(Options::NO_SCROLL) {
        render_scrollbars(ctx, idx, &mut body);
    }
    let padding = ctx.style.padding;
    let scroll = ctx.context_containers()[idx].scroll;
    ctx.push_layout(body.expand(-padding), scroll);
    ctx.context_containers()[idx].body = body;
}

fn render_scrollbars(ctx: &mut Context, idx: usize, body: &mut Rect) {
    let size = ctx.style.scroll_size;

    // Fix E0503: compute content before any borrow of ctx.style.padding mixed with containers
    let content = {
        let padding = ctx.style.padding;
        let c = &ctx.context_containers()[idx];
        Vec2::new(
            c.content.x + padding * 2,
            c.content.y + padding * 2,
        )
    };

    ctx.push_clip(*body);

    let viewport_h = body.h;
    let viewport_w = body.w;

    if content.y > viewport_h {
        body.w -= size;
    }
    if content.x > viewport_w {
        body.h -= size;
    }

    render_scrollbar(ctx, idx, body, content, true);
    render_scrollbar(ctx, idx, body, content, false);
    ctx.pop_clip();
}

fn render_scrollbar(ctx: &mut Context, idx: usize, body: &Rect, content: Vec2, vertical: bool) {
    let size = ctx.style.scroll_size;
    let thumb_min = ctx.style.thumb_size;

    let (content_size, viewport_size) = if vertical {
        (content.y, body.h)
    } else {
        (content.x, body.w)
    };

    let max_scroll = content_size - viewport_size;
    if max_scroll <= 0 {
        if vertical {
            ctx.context_containers()[idx].scroll.y = 0;
        } else {
            ctx.context_containers()[idx].scroll.x = 0;
        }
        return;
    }

    let base = if vertical {
        Rect::new(body.x + body.w, body.y, size, body.h)
    } else {
        Rect::new(body.x, body.y + body.h, body.w, size)
    };

    let bar_id = if vertical { ctx.id_str("!scrollbar_y") } else { ctx.id_str("!scrollbar_x") };
    controls::update(ctx, bar_id, base, Options::empty());

    let scroll = if vertical {
        ctx.context_containers()[idx].scroll.y
    } else {
        ctx.context_containers()[idx].scroll.x
    };

    let visible = if vertical {
        (thumb_min).max(base.h * viewport_size / content_size)
    } else {
        (thumb_min).max(base.w * viewport_size / content_size)
    };

    let mut thumb = base;
    if vertical {
        thumb.h = visible;
        thumb.y += scroll * (base.h - visible) / max_scroll;
    } else {
        thumb.w = visible;
        thumb.x += scroll * (base.w - visible) / max_scroll;
    }

    if ctx.focus == bar_id && ctx.mouse_down == Mouse::Left as u32 {
        let delta = if vertical { ctx.delta.y } else { ctx.delta.x };
        let track = if vertical { base.h } else { base.w };
        let new_scroll = scroll + delta * content_size / track;
        if vertical {
            ctx.context_containers()[idx].scroll.y = new_scroll.clamp(0, max_scroll);
        } else {
            ctx.context_containers()[idx].scroll.x = new_scroll.clamp(0, max_scroll);
        }
    }

    let scroll_color = ctx.style.color(ColorSlot::Scroll);
    let thumb_color = ctx.style.color(ColorSlot::Thumb);
    ctx.draws.rect(base, scroll_color);
    ctx.draws.rect(thumb, thumb_color);

    if body.contains(ctx.mouse) {
        ctx.scroll_target = Some(idx);
    }
}