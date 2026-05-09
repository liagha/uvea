pub struct Region {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

pub struct Atlas {
    pub size: u32,
    pen_x: u32,
    pen_y: u32,
    row_h: u32,
    pub dirty: bool,
    pub pixels: Vec<u8>,
}

impl Atlas {
    pub fn new(size: u32) -> Self {
        Self {
            size,
            pen_x: 0,
            pen_y: 0,
            row_h: 0,
            dirty: false,
            pixels: vec![0u8; (size * size * 4) as usize],
        }
    }

    pub fn alloc(&mut self, w: u32, h: u32) -> Option<(u32, u32)> {
        if self.pen_x + w + 1 > self.size {
            self.pen_x = 0;
            self.pen_y += self.row_h + 1;
            self.row_h = 0;
        }
        if self.pen_y + h + 1 > self.size {
            return None;
        }
        let x = self.pen_x;
        let y = self.pen_y;
        self.pen_x += w + 1;
        if h > self.row_h {
            self.row_h = h;
        }
        Some((x, y))
    }

    pub fn blit_rgba(&mut self, x: u32, y: u32, w: u32, h: u32, src: &[u8], stride: usize) {
        for row in 0..h as usize {
            let dst_off = ((y as usize + row) * self.size as usize + x as usize) * 4;
            let src_off = row * stride;
            self.pixels[dst_off..dst_off + w as usize * 4]
                .copy_from_slice(&src[src_off..src_off + w as usize * 4]);
        }
        self.dirty = true;
    }

    pub fn blit_alpha(&mut self, x: u32, y: u32, w: u32, h: u32, src: &[u8]) {
        for row in 0..h as usize {
            for col in 0..w as usize {
                let dst_off = ((y as usize + row) * self.size as usize + x as usize + col) * 4;
                let a = src[row * w as usize + col];
                self.pixels[dst_off] = 255;
                self.pixels[dst_off + 1] = 255;
                self.pixels[dst_off + 2] = 255;
                self.pixels[dst_off + 3] = a;
            }
        }
        self.dirty = true;
    }

    pub fn reset(&mut self) {
        self.pen_x = 0;
        self.pen_y = 0;
        self.row_h = 0;
        self.pixels.fill(0);
        self.dirty = true;
    }
}