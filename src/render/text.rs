use std::collections::HashMap;
use fontdue::{Font, FontSettings};
use rustybuzz::{Face, UnicodeBuffer, Direction, Script, GlyphBuffer};
use unicode_bidi::BidiInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub id: u16,
    pub size: u16,
}

#[derive(Debug, Clone)]
pub struct GlyphEntry {
    pub atlas_x: u32,
    pub atlas_y: u32,
    pub w: u32,
    pub h: u32,
    pub bearing_x: i32,
    pub bearing_y: i32,
    pub advance: i32,
}

pub struct ShapedGlyph {
    pub id: u16,
    pub x_offset: i32,
    pub y_offset: i32,
    pub x_advance: i32,
}

pub struct Shaped {
    pub glyphs: Vec<ShapedGlyph>,
    pub width: i32,
}

pub struct Shaper {
    face_data: Vec<u8>,
    raster: Font,
    pub size_px: u32,
    pub line_height: u32,
    cache: HashMap<GlyphKey, GlyphEntry>,
}

impl Shaper {
    pub fn from_bytes(data: Vec<u8>, size_px: u32) -> Option<Self> {
        let raster = Font::from_bytes(data.as_slice(), FontSettings::default()).ok()?;
        let metrics = raster.horizontal_line_metrics(size_px as f32)?;
        let line_height = (metrics.ascent - metrics.descent + metrics.line_gap) as u32;

        Some(Self {
            face_data: data,
            raster,
            size_px,
            line_height,
            cache: HashMap::new(),
        })
    }

    pub fn shape(&self, text: &str) -> Shaped {
        let bidi = BidiInfo::new(text, None);
        let mut glyphs = Vec::new();
        let mut total = 0i32;

        for para in &bidi.paragraphs {
            let (levels, runs) = bidi.visual_runs(para, para.range.clone());

            for run in runs {
                let level = levels[run.start];
                let chunk = &text[run.clone()];
                let dir = if level.is_rtl() {
                    Direction::RightToLeft
                } else {
                    Direction::LeftToRight
                };

                let script = detect_script(chunk);
                let mut buf = UnicodeBuffer::new();
                buf.push_str(chunk);
                buf.set_direction(dir);
                buf.set_script(script.unwrap());

                if let Some(face) = Face::from_slice(&self.face_data, 0) {
                    let shaped: GlyphBuffer = rustybuzz::shape(&face, &[], buf);
                    let info = shaped.glyph_infos();
                    let positions = shaped.glyph_positions();
                    let scale = self.size_px as f64 / face.units_per_em() as f64;

                    for (gi, gp) in info.iter().zip(positions.iter()) {
                        let adv = (gp.x_advance as f64 * scale) as i32;
                        glyphs.push(ShapedGlyph {
                            id: gi.glyph_id as u16,
                            x_offset: (gp.x_offset as f64 * scale) as i32,
                            y_offset: (gp.y_offset as f64 * scale) as i32,
                            x_advance: adv,
                        });
                        total += adv;
                    }
                }
            }
        }

        Shaped { glyphs, width: total }
    }

    pub fn rasterize(&mut self, glyph_id: u16) -> Option<(Vec<u8>, u32, u32, i32, i32)> {
        // fontdue 0.9: rasterize_indexed takes u16, not usize
        let (metrics, bitmap) = self.raster.rasterize_indexed(glyph_id, self.size_px as f32);
        if metrics.width == 0 || metrics.height == 0 {
            return None;
        }
        let w = metrics.width as u32;
        let h = metrics.height as u32;
        let bx = metrics.xmin;
        let by = metrics.ymin + metrics.height as i32;
        Some((bitmap, w, h, bx, by))
    }

    pub fn measure(&self, text: &str) -> i32 {
        self.shape(text).width
    }

    pub fn cache(&self) -> &HashMap<GlyphKey, GlyphEntry> {
        &self.cache
    }

    pub fn cache_mut(&mut self) -> &mut HashMap<GlyphKey, GlyphEntry> {
        &mut self.cache
    }
}

fn detect_script(text: &str) -> Option<Script> {
    for ch in text.chars() {
        let cp = ch as u32;
        if (0x0600..=0x06FF).contains(&cp) || (0x0750..=0x077F).contains(&cp) {
            // rustybuzz 0.20: Script constants use OpenType 4-char tags
            return Script::from_iso15924_tag(rustybuzz::ttf_parser::Tag::from_bytes(b"arab"));
        }
        if (0x0400..=0x04FF).contains(&cp) {
            return Script::from_iso15924_tag(rustybuzz::ttf_parser::Tag::from_bytes(b"cyrl"));
        }
        if (0x4E00..=0x9FFF).contains(&cp) {
            return Script::from_iso15924_tag(rustybuzz::ttf_parser::Tag::from_bytes(b"hani"));
        }
        if (0x0900..=0x097F).contains(&cp) {
            return Script::from_iso15924_tag(rustybuzz::ttf_parser::Tag::from_bytes(b"deva"));
        }
    }
    Script::from_iso15924_tag(rustybuzz::ttf_parser::Tag::from_bytes(b"latn"))
}