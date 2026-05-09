/// demo/gallery.rs — Rust port of demo/gallery.c
///
/// Reproduces all three windows from the C demo:
///   • "Demo Window"  – window info, buttons, tree+text, background color, image
///   • "Log Window"   – scrolling log, text-box submit
///   • "Style Editor" – per-color-slot RGBA sliders
///
/// Call `Gallery::build(ctx)` once per frame between begin_frame / end_frame.
use std::{error::Error, sync::Arc};

use uvea::{context::Context, controls, render::Renderer, types::*, window as ui_window};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, Size},
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
    window::{Window, WindowAttributes, WindowId},
};

// ── state ────────────────────────────────────────────────────────────────────

pub struct Gallery {
    /// Scrolling log text
    log: String,
    /// Sync flag: scroll log panel to bottom next frame
    sync: bool,
    /// RGB for the background canvas preview
    canvas: [f32; 3],
    /// Three checkbox states
    checks: [bool; 3],
    /// Textbox buffer for the log submit field
    submit: String,
    /// Textarea buffer (multi-line note)
    note: String,
    /// Optional loaded image handle (Image = usize)
    picture: Option<Image>,
}

impl Default for Gallery {
    fn default() -> Self {
        Self {
            log: String::new(),
            sync: false,
            canvas: [90.0, 95.0, 100.0],
            checks: [true, false, true],
            submit: String::new(),
            note: String::new(),
            picture: None,
        }
    }
}

impl Gallery {
    pub fn new() -> Self {
        Self::default()
    }

    /// Optionally attach a pre-loaded image.
    pub fn set_picture(&mut self, img: Image) {
        self.picture = Some(img);
    }

    fn log_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        if !self.log.is_empty() {
            self.log.push('\n');
        }
        self.log.push_str(text);
        self.sync = true;
    }

    // ── top-level frame ──────────────────────────────────────────────────────

    pub fn build(&mut self, ctx: &mut Context) {
        // Fixed positions matching the C demo
        let demo_rect = Rect::new(40, 40, 300, 450);
        let log_rect = Rect::new(350, 40, 300, 200);
        let style_rect = Rect::new(350, 250, 300, 240);

        // Demo Window
        if ui_window::begin_window(ctx, "Demo Window", demo_rect, Options::empty())
            != Response::empty()
        {
            self.build_demo(ctx);
            ui_window::end_window(ctx);
        }

        // Log Window
        if ui_window::begin_window(ctx, "Log Window", log_rect, Options::empty())
            != Response::empty()
        {
            self.build_log(ctx);
            ui_window::end_window(ctx);
        }

        // Style Editor
        if ui_window::begin_window(ctx, "Style Editor", style_rect, Options::empty())
            != Response::empty()
        {
            self.build_style(ctx);
            ui_window::end_window(ctx);
        }
    }

    // ── Demo Window sections ─────────────────────────────────────────────────

    fn build_demo(&mut self, ctx: &mut Context) {
        self.section_info(ctx);
        self.section_buttons(ctx);
        self.section_tree(ctx);
        self.section_canvas(ctx);
        self.section_picture(ctx);
    }

    /// "Window Info" — shows position & size of the current container
    fn section_info(&self, ctx: &mut Context) {
        if ui_window::begin_tree(ctx, "Window Info", Options::empty()) == Response::empty() {
            return;
        }
        let bounds = ctx.current_container().bounds;
        ctx.layout().row(2, &[54, -1], 0);
        controls::label(ctx, "Position:");
        controls::label(ctx, &format!("{}, {}", bounds.x, bounds.y));
        controls::label(ctx, "Size:");
        controls::label(ctx, &format!("{}, {}", bounds.w, bounds.h));
        ui_window::end_tree(ctx);
    }

    /// "Test Buttons" section
    fn section_buttons(&mut self, ctx: &mut Context) {
        if ui_window::begin_tree(ctx, "Test Buttons", Options::IS_EXPANDED) == Response::empty() {
            return;
        }
        ctx.layout().row(3, &[86, -110, -1], 0);
        controls::label(ctx, "Test buttons 1:");
        if controls::button(ctx, "Button 1", None, Options::empty()) != Response::empty() {
            self.log_text("Pressed button 1");
        }
        if controls::button(ctx, "Button 2", None, Options::empty()) != Response::empty() {
            self.log_text("Pressed button 2");
        }

        controls::label(ctx, "Test buttons 2:");
        if controls::button(ctx, "Button 3", None, Options::empty()) != Response::empty() {
            self.log_text("Pressed button 3");
        }
        if controls::button(ctx, "Popup", None, Options::empty()) != Response::empty() {
            ui_window::open_popup(ctx, "Test Popup");
        }

        if ui_window::begin_popup(ctx, "Test Popup") != Response::empty() {
            controls::button(ctx, "Hello", None, Options::empty());
            controls::button(ctx, "World", None, Options::empty());
            ui_window::end_popup(ctx);
        }
        ui_window::end_tree(ctx);
    }

    /// "Tree and Text" section
    fn section_tree(&mut self, ctx: &mut Context) {
        if ui_window::begin_tree(ctx, "Tree and Text", Options::IS_EXPANDED) == Response::empty() {
            return;
        }
        ctx.layout().row(2, &[140, -1], 0);

        // Left column – nested trees
        ui_window::begin_panel(ctx, "tree_col", Options::NO_FRAME);
        if ui_window::begin_tree(ctx, "Test 1", Options::empty()) != Response::empty() {
            if ui_window::begin_tree(ctx, "Test 1a", Options::empty()) != Response::empty() {
                controls::label(ctx, "Hello");
                controls::label(ctx, "world");
                ui_window::end_tree(ctx);
            }
            if ui_window::begin_tree(ctx, "Test 1b", Options::empty()) != Response::empty() {
                if controls::button(ctx, "Button 1", None, Options::empty()) != Response::empty() {
                    self.log_text("Pressed button 1");
                }
                if controls::button(ctx, "Button 2", None, Options::empty()) != Response::empty() {
                    self.log_text("Pressed button 2");
                }
                ui_window::end_tree(ctx);
            }
            ui_window::end_tree(ctx);
        }
        if ui_window::begin_tree(ctx, "Test 2", Options::empty()) != Response::empty() {
            ctx.layout().row(2, &[54, 54], 0);
            for i in 3..=6 {
                if controls::button(ctx, &format!("Button {i}"), None, Options::empty())
                    != Response::empty()
                {
                    self.log_text(&format!("Pressed button {i}"));
                }
            }
            ui_window::end_tree(ctx);
        }
        if ui_window::begin_tree(ctx, "Test 3", Options::empty()) != Response::empty() {
            controls::checkbox(ctx, "Checkbox 1", &mut self.checks[0]);
            controls::checkbox(ctx, "Checkbox 2", &mut self.checks[1]);
            controls::checkbox(ctx, "Checkbox 3", &mut self.checks[2]);
            ui_window::end_tree(ctx);
        }
        ui_window::end_panel(ctx);

        // Right column – lorem ipsum + textarea
        ui_window::begin_panel(ctx, "text_col", Options::NO_FRAME);
        ctx.layout().row(1, &[-1], 0);
        controls::label(
            ctx,
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
             Maecenas lacinia, sem eu lacinia molestie, mi risus faucibus \
             ipsum, eu varius magna felis a nulla.",
        );
        ctx.layout().row(1, &[-1], 100);
        controls::textbox(ctx, &mut self.note, 1024, Options::MULTILINE);
        ui_window::end_panel(ctx);

        ui_window::end_tree(ctx);
    }

    /// "Background Color" — RGB sliders + colour swatch
    fn section_canvas(&mut self, ctx: &mut Context) {
        if ui_window::begin_tree(ctx, "Background Color", Options::IS_EXPANDED) == Response::empty()
        {
            return;
        }
        ctx.layout().row(2, &[-78, -1], 74);

        // Sliders column
        ui_window::begin_panel(ctx, "canvas_sliders", Options::NO_FRAME);
        ctx.layout().row(2, &[46, -1], 0);
        controls::label(ctx, "Red:");
        controls::slider(
            ctx,
            &mut self.canvas[0],
            0.0,
            255.0,
            1.0,
            "%.0f",
            Options::empty(),
        );
        controls::label(ctx, "Green:");
        controls::slider(
            ctx,
            &mut self.canvas[1],
            0.0,
            255.0,
            1.0,
            "%.0f",
            Options::empty(),
        );
        controls::label(ctx, "Blue:");
        controls::slider(
            ctx,
            &mut self.canvas[2],
            0.0,
            255.0,
            1.0,
            "%.0f",
            Options::empty(),
        );
        ui_window::end_panel(ctx);

        // Colour swatch
        let area = ctx.next_bounds();
        let swatch_color = Color::rgba(
            self.canvas[0] as u8,
            self.canvas[1] as u8,
            self.canvas[2] as u8,
            255,
        );
        ctx.draws.rect(area, swatch_color);
        let label = format!(
            "#{:02X}{:02X}{:02X}",
            self.canvas[0] as u8, self.canvas[1] as u8, self.canvas[2] as u8,
        );
        let font = ctx.style.font;
        let text_color = ctx.style.color(ColorSlot::Text);
        let pos = Vec2::new(area.x + ctx.style.padding, area.y + area.h / 4);
        ctx.draws.text(font, label, pos, text_color);

        ui_window::end_tree(ctx);
    }

    /// "Image Render" — show picture or placeholder
    fn section_picture(&self, ctx: &mut Context) {
        if ui_window::begin_tree(ctx, "Image Render", Options::IS_EXPANDED) == Response::empty() {
            return;
        }
        ctx.layout().row(1, &[-1], 0);
        match self.picture {
            Some(img) => {
                let bounds = ctx.next_bounds();
                let tint = Color::WHITE;
                ctx.draws.image(img, bounds, tint);
            }
            None => {
                controls::label(ctx, "Missing picture");
            }
        }
        ui_window::end_tree(ctx);
    }

    // ── Log Window ───────────────────────────────────────────────────────────

    fn build_log(&mut self, ctx: &mut Context) {
        // Scrolling log panel (takes all height except bottom row)
        ctx.layout().row(1, &[-1], -25);
        ui_window::begin_panel(ctx, "Log Output", Options::empty());

        ctx.layout().row(1, &[-1], -1);
        controls::label(ctx, &self.log.clone());

        if self.sync {
            // Scroll to bottom: set scroll.y to content height
            let content_y = ctx.current_container().content.y;
            ctx.current_container_mut().scroll.y = content_y;
            self.sync = false;
        }

        ui_window::end_panel(ctx);

        // Submit row
        let mut done = false;
        ctx.layout().row(2, &[-70, -1], 0);
        if controls::textbox(ctx, &mut self.submit, 128, Options::empty())
            .contains(Response::SUBMIT)
        {
            done = true;
        }
        if controls::button(ctx, "Submit", None, Options::empty()) != Response::empty() {
            done = true;
        }
        if done && !self.submit.is_empty() {
            let text = self.submit.clone();
            self.log_text(&text);
            self.submit.clear();
        }
    }

    // ── Style Editor ─────────────────────────────────────────────────────────

    fn build_style(&mut self, ctx: &mut Context) {
        // Each row: label | R slider | G slider | B slider | A slider | swatch
        let body_w = ctx.current_container().body.w;
        let span = (body_w as f32 * 0.14) as i32;
        ctx.layout().row(6, &[80, span, span, span, span, -1], 0);

        let slots: &[(ColorSlot, &str)] = &[
            (ColorSlot::Text, "text:"),
            (ColorSlot::Border, "border:"),
            (ColorSlot::Window, "windowbg:"),
            (ColorSlot::Title, "titlebg:"),
            (ColorSlot::Heading, "titletext:"),
            (ColorSlot::Panel, "panelbg:"),
            (ColorSlot::Button, "button:"),
            (ColorSlot::ButtonHover, "buttonhover:"),
            (ColorSlot::ButtonFocus, "buttonfocus:"),
            (ColorSlot::Input, "base:"),
            (ColorSlot::InputHover, "basehover:"),
            (ColorSlot::InputFocus, "basefocus:"),
            (ColorSlot::Scroll, "scrollbase:"),
            (ColorSlot::Thumb, "scrollthumb:"),
            (ColorSlot::Selection, "selection:"),
        ];

        for (slot, name) in slots {
            controls::label(ctx, name);
            let idx = *slot as usize;

            // R
            let mut r = ctx.style.colors[idx].r as f32;
            controls::slider(ctx, &mut r, 0.0, 255.0, 1.0, "%.0f", Options::ALIGN_CENTER);
            ctx.style.colors[idx].r = r as u8;

            // G
            let mut g = ctx.style.colors[idx].g as f32;
            controls::slider(ctx, &mut g, 0.0, 255.0, 1.0, "%.0f", Options::ALIGN_CENTER);
            ctx.style.colors[idx].g = g as u8;

            // B
            let mut b = ctx.style.colors[idx].b as f32;
            controls::slider(ctx, &mut b, 0.0, 255.0, 1.0, "%.0f", Options::ALIGN_CENTER);
            ctx.style.colors[idx].b = b as u8;

            // A
            let mut a = ctx.style.colors[idx].a as f32;
            controls::slider(ctx, &mut a, 0.0, 255.0, 1.0, "%.0f", Options::ALIGN_CENTER);
            ctx.style.colors[idx].a = a as u8;

            // Swatch
            let swatch_rect = ctx.next_bounds();
            let swatch_color = ctx.style.colors[idx];
            ctx.draws.rect(swatch_rect, swatch_color);
        }
    }
}
struct GalleryApp {
    ctx: Context,
    gallery: Gallery,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    modifiers: ModifiersState,
}

impl GalleryApp {
    fn new() -> Self {
        Self {
            ctx: Context::new(),
            gallery: Gallery::new(),
            window: None,
            renderer: None,
            modifiers: ModifiersState::empty(),
        }
    }

    fn set_key(&mut self, key: Key, pressed: bool) {
        if pressed {
            self.ctx.key_down(key as u32);
        } else {
            self.ctx.key_up(key as u32);
        }
    }

    fn update_modifiers(&mut self, modifiers: ModifiersState) {
        if modifiers.shift_key() != self.modifiers.shift_key() {
            self.set_key(Key::Shift, modifiers.shift_key());
        }
        if modifiers.control_key() != self.modifiers.control_key() {
            self.set_key(Key::Control, modifiers.control_key());
        }
        if modifiers.alt_key() != self.modifiers.alt_key() {
            self.set_key(Key::Alt, modifiers.alt_key());
        }
        self.modifiers = modifiers;
    }

    fn render_frame(&mut self) {
        self.ctx.begin_frame();
        self.gallery.build(&mut self.ctx);
        self.ctx.end_frame();

        if let Some(renderer) = self.renderer.as_mut() {
            renderer.render(self.ctx.draws.iter().cloned(), Color::rgba(30, 30, 30, 255));
        }
    }
}

impl ApplicationHandler for GalleryApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = WindowAttributes::default()
            .with_title("uvea Gallery")
            .with_inner_size(Size::Logical(LogicalSize::new(700.0, 540.0)));
        let window = Arc::new(event_loop.create_window(attrs).expect("create window"));
        let size = window.inner_size();

        // Use the winit window directly to create the surface
        // Instead of creating a new wgpu instance, derive the surface from winit
        let renderer = pollster::block_on(Renderer::new_with_window(
            window.clone(),
            size.width,
            size.height,
            load_font_data(),
            16,
        ));

        self.renderer = Some(renderer);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => self.render_frame(),
            WindowEvent::CursorMoved { position, .. } => {
                self.ctx.mouse_move(position.x as i32, position.y as i32);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(mouse) = map_mouse_button(button) {
                    let pos = self.ctx.mouse;
                    match state {
                        ElementState::Pressed => self.ctx.mouse_down(pos.x, pos.y, mouse),
                        ElementState::Released => self.ctx.mouse_up(pos.x, pos.y, mouse),
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(x, y) => {
                    self.ctx.scroll((x * 30.0) as i32, (y * 30.0) as i32)
                }
                MouseScrollDelta::PixelDelta(pos) => self.ctx.scroll(pos.x as i32, pos.y as i32),
            },
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    if let Some(key) = map_key_code(code) {
                        self.set_key(key, event.state == ElementState::Pressed);
                    }
                }
                if event.state == ElementState::Pressed && !self.modifiers.control_key() {
                    if let Some(text) = event.text {
                        self.ctx.text_input(&text);
                    }
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => self.update_modifiers(modifiers.state()),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

fn map_mouse_button(button: MouseButton) -> Option<u32> {
    match button {
        MouseButton::Left => Some(Mouse::Left as u32),
        MouseButton::Right => Some(Mouse::Right as u32),
        MouseButton::Middle => Some(Mouse::Middle as u32),
        _ => None,
    }
}

fn map_key_code(code: KeyCode) -> Option<Key> {
    match code {
        KeyCode::Backspace => Some(Key::Backspace),
        KeyCode::Enter | KeyCode::NumpadEnter => Some(Key::Return),
        KeyCode::ArrowLeft => Some(Key::Left),
        KeyCode::ArrowRight => Some(Key::Right),
        KeyCode::ArrowUp => Some(Key::Up),
        KeyCode::ArrowDown => Some(Key::Down),
        KeyCode::Delete => Some(Key::Delete),
        KeyCode::KeyA => Some(Key::A),
        KeyCode::KeyC => Some(Key::C),
        KeyCode::KeyV => Some(Key::V),
        KeyCode::KeyX => Some(Key::X),
        _ => None,
    }
}

fn load_font_data() -> Vec<u8> {
    const CANDIDATES: &[&str] = &[
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf",
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "C:\\Windows\\Fonts\\arial.ttf",
    ];

    CANDIDATES
        .iter()
        .find_map(|path| std::fs::read(path).ok())
        .unwrap_or_default()
}

fn main() -> std::result::Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = GalleryApp::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
