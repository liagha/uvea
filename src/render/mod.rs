pub mod atlas;
pub mod pipeline;
pub mod text;
pub mod vertex;

use std::sync::Arc;
use wgpu::*;
use winit::window::Window;
use crate::{
    draw::Command,
    render::{
        atlas::Atlas,
        pipeline::Pipeline,
        text::{GlyphEntry, GlyphKey, Shaper},
        vertex::Batch,
    },
    types::{Color, Icon, Rect, Vec2},
};

pub struct Renderer {
    device: Device,
    queue: Queue,
    adapter: Adapter,
    surface: Surface<'static>,
    format: TextureFormat,
    pipeline: Pipeline,
    atlas: Atlas,
    batch: Batch,
    shaper: Option<Shaper>,
    width: u32,
    height: u32,
}

impl Renderer {
    pub async fn new(
        surface: Surface<'static>,
        width: u32,
        height: u32,
        font_data: Vec<u8>,
        font_size: u32,
    ) -> Self {
        let instance = Instance::default();
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("no adapter");

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default())
            .await
            .expect("no device");

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        surface.configure(&device, &SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        });

        let atlas_size = 2048u32;
        let pipeline = Pipeline::new(&device, format, atlas_size);
        let atlas = Atlas::new(atlas_size);
        let shaper = Shaper::from_bytes(font_data, font_size);

        Self {
            device,
            queue,
            adapter,
            surface,
            format,
            pipeline,
            atlas,
            batch: Batch::new(),
            shaper,
            width,
            height,
        }
    }

    // In src/render/mod.rs, modify the new_with_window method:

    // In src/render/mod.rs, update the new_with_window method:

    pub async fn new_with_window(
        window: Arc<Window>,
        width: u32,
        height: u32,
        font_data: Vec<u8>,
        font_size: u32,
    ) -> Self {
        let instance = wgpu::Instance::default();

        let surface = instance.create_surface(window).expect("create surface");

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("no adapter");

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                    label: None,
                    memory_hints: MemoryHints::default(),
                    experimental_features: ExperimentalFeatures::default(),
                    trace: Trace::Off,
                },
            )
            .await
            .expect("no device");

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        surface.configure(&device, &SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        });

        let atlas_size = 2048u32;
        let pipeline = Pipeline::new(&device, format, atlas_size);
        let atlas = Atlas::new(atlas_size);
        let shaper = Shaper::from_bytes(font_data, font_size);

        Self {
            device,
            queue,
            adapter,
            surface,
            format,
            pipeline,
            atlas,
            batch: Batch::new(),
            shaper,
            width,
            height,
        }
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 { return; }
        self.width = width;
        self.height = height;
        let caps = self.surface.get_capabilities(&self.adapter);
        self.surface.configure(&self.device, &SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: self.format,
            width,
            height,
            present_mode: PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        });
    }

    pub fn render(&mut self, commands: impl Iterator<Item = Command>, clear: Color) {
        // In wgpu 29, get_current_texture returns CurrentSurfaceTexture enum
        // Need to match on it to handle different states
        use wgpu::CurrentSurfaceTexture;

        // In src/render/mod.rs, replace lines 113-117 with:

        let frame = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(texture) => texture,
            CurrentSurfaceTexture::Suboptimal(texture) => texture,
            CurrentSurfaceTexture::Timeout => {
                // Skip this frame
                return;
            }
            CurrentSurfaceTexture::Occluded => {
                // Skip this frame, window is minimized/occluded
                return;
            }
            CurrentSurfaceTexture::Outdated => {
                // Surface configuration is outdated, reconfigure and try again
                self.resize(self.width, self.height);
                return;
            }
            CurrentSurfaceTexture::Lost => {
                // Surface lost, needs recreation
                eprintln!("Surface lost, needs recreation");
                return;
            }
            CurrentSurfaceTexture::Validation => {
                // Validation error occurred
                eprintln!("Validation error in get_current_texture");
                return;
            }
        };

        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        self.batch.clear();
        self.pipeline.update_projection(&self.queue, self.width as f32, self.height as f32);

        for cmd in commands {
            match cmd {
                Command::Clip(_) => {}
                Command::Rect { bounds, color, radius, outline } => {
                    let c = [color.r, color.g, color.b, color.a];
                    let mode = if radius <= 0.0 { 0.0 } else if outline { 3.0 } else { 1.0 };
                    let hw = bounds.w as f32 * 0.5;
                    let hh = bounds.h as f32 * 0.5;
                    self.batch.push_quad(
                        bounds.x as f32, bounds.y as f32,
                        bounds.w as f32, bounds.h as f32,
                        0.0, 0.0, 1.0, 1.0,
                        hw, hh, radius, mode, c,
                    );
                }
                Command::Text { text, position, color, .. } => {
                    self.draw_text(&text, position, color);
                }
                Command::Icon { kind, bounds, color } => {
                    self.draw_icon(kind, bounds, color);
                }
                Command::Image { bounds, tint, .. } => {
                    let c = [tint.r, tint.g, tint.b, tint.a];
                    self.batch.push_quad(
                        bounds.x as f32, bounds.y as f32,
                        bounds.w as f32, bounds.h as f32,
                        0.0, 0.0, 1.0, 1.0,
                        0.0, 0.0, 0.0, 0.0, c,
                    );
                }
            }
        }

        if self.atlas.dirty {
            self.pipeline.update_atlas(&self.queue, &self.atlas.pixels);
            self.atlas.dirty = false;
        }

        if !self.batch.data().is_empty() {
            let data = bytemuck::cast_slice(self.batch.data());
            self.queue.write_buffer(&self.pipeline.vertex_buf, 0, data);
        }

        let wgpu_clear = wgpu::Color {
            r: clear.r as f64 / 255.0,
            g: clear.g as f64 / 255.0,
            b: clear.b as f64 / 255.0,
            a: clear.a as f64 / 255.0,
        };

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu_clear),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
            pass.set_pipeline(&self.pipeline.pipeline);
            pass.set_bind_group(0, &self.pipeline.bind_group, &[]);
            pass.set_vertex_buffer(0, self.pipeline.vertex_buf.slice(..));
            pass.draw(0..self.batch.len() as u32, 0..1);
        }
        self.queue.submit([encoder.finish()]);
        frame.present();
    }

    fn draw_text(&mut self, text: &str, position: Vec2, color: Color) {
        let Some(shaper) = &mut self.shaper else { return; };
        let shaped = shaper.shape(text);
        let size_key = shaper.size_px as u16;
        let mut cursor_x = position.x;

        for glyph in &shaped.glyphs {
            let key = GlyphKey { id: glyph.id, size: size_key };

            if !shaper.cache().contains_key(&key) {
                if let Some((pixels, w, h, bx, by)) = shaper.rasterize(glyph.id) {
                    if let Some((ax, ay)) = self.atlas.alloc(w, h) {
                        self.atlas.blit_alpha(ax, ay, w, h, &pixels);
                        shaper.cache_mut().insert(key, GlyphEntry {
                            atlas_x: ax,
                            atlas_y: ay,
                            w,
                            h,
                            bearing_x: bx,
                            bearing_y: by,
                            advance: glyph.x_advance,
                        });
                    }
                }
            }

            if let Some(entry) = shaper.cache().get(&key).cloned() {
                let x = cursor_x + entry.bearing_x + glyph.x_offset;
                let y = position.y - entry.bearing_y + glyph.y_offset;
                let s = self.pipeline.atlas_size as f32;
                let u0 = entry.atlas_x as f32 / s;
                let v0 = entry.atlas_y as f32 / s;
                let u1 = (entry.atlas_x + entry.w) as f32 / s;
                let v1 = (entry.atlas_y + entry.h) as f32 / s;

                self.batch.push_quad(
                    x as f32, y as f32, entry.w as f32, entry.h as f32,
                    u0, v0, u1, v1,
                    0.0, 0.0, 0.0, 0.0,
                    [color.r, color.g, color.b, color.a],
                );
            }

            cursor_x += glyph.x_advance;
        }
    }

    fn draw_icon(&mut self, kind: Icon, bounds: Rect, color: Color) {
        let cx = bounds.x + bounds.w / 2;
        let cy = bounds.y + bounds.h / 2;
        let c: [u8; 4] = [color.r, color.g, color.b, color.a];
        match kind {
            Icon::Close => {
                self.line(cx - 4, cy - 4, cx + 4, cy + 4, c);
                self.line(cx + 4, cy - 4, cx - 4, cy + 4, c);
            }
            Icon::Check => {
                self.line(cx - 3, cy, cx - 1, cy + 3, c);
                self.line(cx - 1, cy + 3, cx + 4, cy - 3, c);
            }
            Icon::Collapsed => self.tri(cx - 3, cy - 4, cx - 3, cy + 4, cx + 4, cy, c),
            Icon::Expanded => self.tri(cx - 4, cy - 3, cx + 4, cy - 3, cx, cy + 4, c),
        }
    }

    fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: [u8; 4]) {
        let dx = (x2 - x1) as f32;
        let dy = (y2 - y1) as f32;
        let len = (dx * dx + dy * dy).sqrt().max(0.001);
        let nx = dy / len;
        let ny = -dx / len;

        let corners = [
            (x1 as f32 + nx, y1 as f32 + ny),
            (x1 as f32 - nx, y1 as f32 - ny),
            (x2 as f32 - nx, y2 as f32 - ny),
            (x2 as f32 + nx, y2 as f32 + ny),
        ];

        self.batch.push_quad(
            corners[0].0, corners[0].1,
            corners[2].0 - corners[0].0, corners[2].1 - corners[0].1,
            0.5, 0.5, 0.5, 0.5,
            0.0, 0.0, 0.0, 0.0,
            color,
        );
    }

    fn tri(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32, color: [u8; 4]) {
        let cx = (x1 + x2 + x3) as f32 / 3.0;
        let cy = (y1 + y2 + y3) as f32 / 3.0;
        let hw = ((x1.max(x2).max(x3) - x1.min(x2).min(x3)) as f32) * 0.5;
        let hh = ((y1.max(y2).max(y3) - y1.min(y2).min(y3)) as f32) * 0.5;
        self.batch.push_quad(
            cx - hw, cy - hh, hw * 2.0, hh * 2.0,
            0.5, 0.5, 0.5, 0.5,
            0.0, 0.0, 0.0, 0.0,
            color,
        );
    }
}