// examples/gallery.rs

use uvea::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color as SdlColor;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::Instant;

struct State {
    log: [u8; 64000],
    sync: bool,
    canvas: [f32; 3],
    checks: [bool; 3],
    submit: [u8; 128],
    note: [u8; 1024],
    picture: Image,
}

struct WindowDef {
    name: &'static str,
    bounds: Rectangle,
    flags: i32,
    build: fn(&mut Context, &mut State),
}

struct Section {
    name: &'static str,
    flags: i32,
    build: fn(&mut Context, &mut State),
}

struct Theme {
    name: &'static str,
    tint: usize,
}

const COLORS: [Theme; 15] = [
    Theme { name: "text:",        tint: 0  },
    Theme { name: "border:",      tint: 1  },
    Theme { name: "windowbg:",    tint: 2  },
    Theme { name: "titlebg:",     tint: 3  },
    Theme { name: "titletext:",   tint: 4  },
    Theme { name: "panelbg:",     tint: 5  },
    Theme { name: "button:",      tint: 6  },
    Theme { name: "buttonhover:", tint: 7  },
    Theme { name: "buttonfocus:", tint: 8  },
    Theme { name: "base:",        tint: 9  },
    Theme { name: "basehover:",   tint: 10 },
    Theme { name: "basefocus:",   tint: 11 },
    Theme { name: "scrollbase:",  tint: 12 },
    Theme { name: "scrollthumb:", tint: 13 },
    Theme { name: "selection:",   tint: 14 },
];

fn log_text(state: &mut State, string: &str) {
    if string.is_empty() {
        return;
    }
    let used = strlen(&state.log);
    let mut pos = used;
    if used > 0 {
        if pos + 1 < state.log.len() {
            state.log[pos] = b'\n';
            pos += 1;
        }
    }
    let bytes = string.as_bytes();
    let available = state.log.len().saturating_sub(pos).saturating_sub(1);
    let copy_len = bytes.len().min(available);
    state.log[pos..pos + copy_len].copy_from_slice(&bytes[..copy_len]);
    if pos + copy_len < state.log.len() {
        state.log[pos + copy_len] = 0;
    }
    state.sync = true;
}

fn edit_color(context: &mut Context, tint: &mut u8, low: Real, high: Real) -> i32 {
    let mut temp = *tint as Real;
    let id_bytes = (tint as *const _ as usize).to_ne_bytes();
    context.push_identifier(&id_bytes);
    let result = context.draw_slider(&mut temp, low, high, 0.0, "%.0f", ALIGN_CENTER);
    *tint = temp as u8;
    context.pop_identifier();
    result
}

fn build_information(context: &mut Context, _state: &mut State) {
    let view = context.current_container_index();
    let bounds = context.container_pool.items[view].bounds;
    let size = Rectangle::new(bounds.width, bounds.height, 0, 0);

    context.layout_row(2, Some(&[54, -1]), 0);
    context.build_label("Position:");
    let pos_text = format!("{}, {}", bounds.x, bounds.y);
    context.build_label(&pos_text);
    context.build_label("Size:");
    let size_text = format!("{}, {}", size.x, size.y);
    context.build_label(&size_text);
}

fn build_actions(context: &mut Context, state: &mut State) {
    context.layout_row(3, Some(&[86, -110, -1]), 0);
    context.build_label("Test buttons 1:");
    if context.draw_button(Some("Button 1"), 0, 0) != 0 {
        log_text(state, "Pressed button 1");
    }
    if context.draw_button(Some("Button 2"), 0, 0) != 0 {
        log_text(state, "Pressed button 2");
    }
    context.build_label("Test buttons 2:");
    if context.draw_button(Some("Button 3"), 0, 0) != 0 {
        log_text(state, "Pressed button 3");
    }
    if context.draw_button(Some("Popup"), 0, 0) != 0 {
        context.open_popup("Test Popup");
    }
    if context.push_popup("Test Popup") != 0 {
        context.draw_button(Some("Hello"), 0, 0);
        context.draw_button(Some("World"), 0, 0);
        context.pop_popup();
    }
}

fn build_tree(context: &mut Context, state: &mut State) {
    context.layout_row(2, Some(&[140, -1]), 0);

    context.begin_column();
    if context.push_tree("Test 1", 0) != 0 {
        if context.push_tree("Test 1a", 0) != 0 {
            context.build_label("Hello");
            context.build_label("world");
            context.pop_tree();
        }
        if context.push_tree("Test 1b", 0) != 0 {
            if context.draw_button(Some("Button 1"), 0, 0) != 0 {
                log_text(state, "Pressed button 1");
            }
            if context.draw_button(Some("Button 2"), 0, 0) != 0 {
                log_text(state, "Pressed button 2");
            }
            context.pop_tree();
        }
        context.pop_tree();
    }
    if context.push_tree("Test 2", 0) != 0 {
        context.layout_row(2, Some(&[54, 54]), 0);
        if context.draw_button(Some("Button 3"), 0, 0) != 0 {
            log_text(state, "Pressed button 3");
        }
        if context.draw_button(Some("Button 4"), 0, 0) != 0 {
            log_text(state, "Pressed button 4");
        }
        if context.draw_button(Some("Button 5"), 0, 0) != 0 {
            log_text(state, "Pressed button 5");
        }
        if context.draw_button(Some("Button 6"), 0, 0) != 0 {
            log_text(state, "Pressed button 6");
        }
        context.pop_tree();
    }
    if context.push_tree("Test 3", 0) != 0 {
        context.draw_checkbox("Checkbox 1", &mut state.checks[0]);
        context.draw_checkbox("Checkbox 2", &mut state.checks[1]);
        context.draw_checkbox("Checkbox 3", &mut state.checks[2]);
        context.pop_tree();
    }
    context.end_column();

    context.begin_column();
    context.layout_row(1, Some(&[-1]), 0);
    context.build_text(
        "Lorem ipsum dolor sit amet, consectetur adipiscing \
         elit. Maecenas lacinia, sem eu lacinia molestie, mi risus faucibus \
         ipsum, eu varius magna felis a nulla.",
    );
    context.layout_row(1, Some(&[-1]), 100);
    context.draw_textbox(&mut state.note, MULTILINE);
    context.end_column();
}

fn build_canvas(context: &mut Context, state: &mut State) {
    context.layout_row(2, Some(&[-78, -1]), 74);
    context.begin_column();
    context.layout_row(2, Some(&[46, -1]), 0);
    context.build_label("Red:");
    context.draw_slider(&mut state.canvas[0], 0.0, 255.0, 0.0, "%.0f", ALIGN_CENTER);
    context.build_label("Green:");
    context.draw_slider(&mut state.canvas[1], 0.0, 255.0, 0.0, "%.0f", ALIGN_CENTER);
    context.build_label("Blue:");
    context.draw_slider(&mut state.canvas[2], 0.0, 255.0, 0.0, "%.0f", ALIGN_CENTER);
    context.end_column();

    let area = context.next_bounds();
    let color = Color::new(
        state.canvas[0] as u8,
        state.canvas[1] as u8,
        state.canvas[2] as u8,
        255,
    );
    draw::draw_rectangle(context, area, color);
    let hex = format!(
        "#{:02X}{:02X}{:02X}",
        state.canvas[0] as u8,
        state.canvas[1] as u8,
        state.canvas[2] as u8
    );
    context.control_text(&hex, area, ColorIndex::Text, ALIGN_CENTER);
}

fn build_picture(context: &mut Context, state: &mut State) {
    if state.picture.is_null() {
        context.layout_row(1, Some(&[-1]), 0);
        context.build_label("Missing picture");
    } else {
        context.layout_row(1, Some(&[-1]), 120);
        context.build_image(state.picture);
    }
}

fn build_demo(context: &mut Context, state: &mut State) {
    let sections = [
        Section { name: "Window Info", flags: 0, build: build_information },
        Section { name: "Test Buttons", flags: IS_EXPANDED, build: build_actions },
        Section { name: "Tree and Text", flags: IS_EXPANDED, build: build_tree },
        Section { name: "Background Color", flags: IS_EXPANDED, build: build_canvas },
        Section { name: "Image Render", flags: IS_EXPANDED, build: build_picture },
    ];

    let idx = context.current_container_index();
    {
        let container = &mut context.container_pool.items[idx];
        container.bounds.width = container.bounds.width.max(240);
        container.bounds.height = container.bounds.height.max(300);
    }

    for section in &sections {
        if context.draw_header(section.name, section.flags) != 0 {
            (section.build)(context, state);
        }
    }
}

fn build_log(context: &mut Context, state: &mut State) {
    context.layout_row(1, Some(&[-1]), -25);
    context.push_panel("Log Output", 0);
    let view_idx = context.current_container_index();
    context.layout_row(1, Some(&[-1]), -1);
    context.build_text(std::str::from_utf8(&state.log).unwrap_or(""));
    context.pop_panel();

    if state.sync {
        context.container_pool.items[view_idx].scroll.y =
            context.container_pool.items[view_idx].content.y;
        state.sync = false;
    }

    context.layout_row(2, Some(&[-70, -1]), 0);
    let mut done = false;
    let result = context.draw_textbox(&mut state.submit, 0);
    if result & RESULT_SUBMIT != 0 {
        context.set_focus(context.last_identifier);
        done = true;
    }
    if context.draw_button(Some("Submit"), 0, 0) != 0 {
        done = true;
    }
    if done && state.submit[0] != 0 {
        let text = {
            let len = strlen(&state.submit);
            std::str::from_utf8(&state.submit[..len]).unwrap_or("").to_owned()
        };
        log_text(state, &text);
        state.submit[0] = 0;
    }
}

fn build_style(context: &mut Context, _state: &mut State) {
    let span = (context.container_pool.items[context.current_container_index()].body.width as f32 * 0.14) as i32;
    context.layout_row(6, Some(&[80, span, span, span, span, -1]), 0);
    for theme in &COLORS {
        let color_index = theme.tint;
        context.build_label(theme.name);
        unsafe {
            let style = &mut * (context.style as *mut Style);
            edit_color(context, &mut style.colors[color_index].red, 0.0, 255.0);
            edit_color(context, &mut style.colors[color_index].green, 0.0, 255.0);
            edit_color(context, &mut style.colors[color_index].blue, 0.0, 255.0);
            edit_color(context, &mut style.colors[color_index].alpha, 0.0, 255.0);
            let bounds = context.next_bounds();
            draw::draw_rectangle(context, bounds, style.colors[color_index]);
        }
    }
}

fn update(context: &mut Context, state: &mut State) {
    let mut windows = [
        WindowDef { name: "Style Editor", bounds: Rectangle::new(350, 250, 300, 240), flags: 0, build: build_style },
        WindowDef { name: "Log Window", bounds: Rectangle::new(350, 40, 300, 200), flags: 0, build: build_log },
        WindowDef { name: "Demo Window", bounds: Rectangle::new(40, 40, 300, 450), flags: 0, build: build_demo },
    ];
    context.begin_frame();
    for window in &mut windows {
        if context.push_window(window.name, window.bounds, window.flags) != 0 {
            (window.build)(context, state);
            context.pop_window();
        }
    }
    context.end_frame();
}

fn strlen(buf: &[u8]) -> usize {
    buf.iter().position(|&b| b == 0).unwrap_or(buf.len())
}

fn map_mouse(button: MouseButton) -> i32 {
    match button {
        MouseButton::Left => MOUSE_LEFT,
        MouseButton::Right => MOUSE_RIGHT,
        MouseButton::Middle => MOUSE_MIDDLE,
        _ => 0,
    }
}

fn map_key(key: Keycode) -> i32 {
    match key {
        Keycode::LShift | Keycode::RShift => KEY_SHIFT,
        Keycode::LCtrl | Keycode::RCtrl => KEY_CONTROL,
        Keycode::LAlt | Keycode::RAlt => KEY_ALT,
        Keycode::Return => KEY_RETURN,
        Keycode::Backspace => KEY_BACKSPACE,
        Keycode::Left => KEY_LEFT,
        Keycode::Right => KEY_RIGHT,
        Keycode::Up => KEY_UP,
        Keycode::Down => KEY_DOWN,
        Keycode::Delete => KEY_DELETE,
        Keycode::A => KEY_A,
        Keycode::C => KEY_C,
        Keycode::V => KEY_V,
        Keycode::X => KEY_X,
        _ => 0,
    }
}

fn measure_width(text: &str, limit: i32) -> i32 {
    let len = if limit < 0 { text.len() } else { limit as usize };
    (len as i32) * 10
}

fn measure_height() -> i32 {
    20
}

fn next_instruction<'a>(
    instructions: &'a [Instruction],
    index: &mut usize,
) -> Option<&'a Instruction> {
    loop {
        if *index >= instructions.len() {
            return None;
        }
        match &instructions[*index] {
            Instruction::Jump { target } => {
                *index = *target;
            }
            other => {
                *index += 1;
                return Some(other);
            }
        }
    }
}

fn render(instructions: &[Instruction], canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(SdlColor::RGB(0, 0, 0));
    canvas.clear();
    let mut idx = 0;
    while let Some(instr) = next_instruction(instructions, &mut idx) {
        match instr {
            Instruction::Rectangle { bounds, color, .. } => {
                let rect = Rect::new(bounds.x, bounds.y, bounds.width as u32, bounds.height as u32);
                canvas.set_draw_color(SdlColor::RGBA(color.red, color.green, color.blue, color.alpha));
                canvas.fill_rect(rect).ok();
            }
            Instruction::Text { .. } => {
            }
            Instruction::Icon { bounds, color, .. } => {
                let rect = Rect::new(bounds.x, bounds.y, bounds.width as u32, bounds.height as u32);
                canvas.set_draw_color(SdlColor::RGBA(color.red, color.green, color.blue, color.alpha));
                canvas.draw_rect(rect).ok();
            }
            Instruction::Clip { bounds } => {
                let rect = Rect::new(bounds.x, bounds.y, bounds.width as u32, bounds.height as u32);
                canvas.set_clip_rect(rect);
            }
            _ => {}
        }
    }
    canvas.set_clip_rect(None);
    canvas.present();
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let benchmark = args.len() >= 2 && args[1] == "--bench";
    let frames: usize = if benchmark {
        if args.len() >= 3 {
            args[2].parse().unwrap_or(1200)
        } else {
            1200
        }
    } else {
        0
    };

    let mut state = State {
        log: [0u8; 64000],
        sync: false,
        canvas: [90.0, 95.0, 100.0],
        checks: [true, false, true],
        submit: [0u8; 128],
        note: [0u8; 1024],
        picture: std::ptr::null_mut(),
    };

    let mut context = Context::new(
        |_, bytes| measure_width(std::str::from_utf8(bytes).unwrap_or(""), -1),
        |_| measure_height(),
    );

    if benchmark {
        let start = Instant::now();
        let mut total_instructions = 0;
        let mut max_instructions = 0;
        for _ in 0..frames {
            update(&mut context, &mut state);
            let mut count = 0;
            let mut idx = 0;
            while let Some(_) = next_instruction(&context.instructions, &mut idx) {
                count += 1;
            }
            total_instructions += count;
            max_instructions = max_instructions.max(count);
        }
        let elapsed = start.elapsed().as_secs_f64();
        let fps = frames as f64 / elapsed;
        let avg_instructions = total_instructions as f64 / frames as f64;
        println!("Benchmark: {} frames", frames);
        println!("  throughput: {:.2} fps", fps);
        println!("  instructions: avg={:.1} max={}", avg_instructions, max_instructions);
        return Ok(());
    }

    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let window = video.window("ceebs gallery", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let mut event_pump = sdl.event_pump()?;
    sdl.mouse().show_cursor(true);

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::MouseMotion { x, y, .. } => {
                    context.input_mouse(x, y);
                }
                Event::MouseWheel { y, .. } => {
                    context.input_scroll(0, y * -30);
                }
                Event::TextInput { text, .. } => {
                    context.input_text(&text);
                }
                Event::MouseButtonDown { x, y, mouse_btn, .. } => {
                    let button = map_mouse(mouse_btn);
                    if button != 0 {
                        context.input_down(x, y, button);
                    }
                }
                Event::MouseButtonUp { x, y, mouse_btn, .. } => {
                    let button = map_mouse(mouse_btn);
                    if button != 0 {
                        context.input_up(x, y, button);
                    }
                }
                Event::KeyDown { keycode: Some(key), .. } => {
                    let action = map_key(key);
                    if action != 0 {
                        context.input_key(action);
                    }
                }
                Event::KeyUp { keycode: Some(key), .. } => {
                    let action = map_key(key);
                    if action != 0 {
                        context.input_keyup(action);
                    }
                }
                _ => {}
            }
        }

        update(&mut context, &mut state);
        render(&context.instructions, &mut canvas);
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    Ok(())
}