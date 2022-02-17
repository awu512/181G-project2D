// We'll make our Color type an RGBA8888 pixel.
pub type Color = (u8, u8, u8, u8);

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub struct Vec2i {
    // Or Vec2f for floats?
    pub x: i32,
    pub y: i32,
}

impl std::ops::Add<Vec2i> for Vec2i {
    type Output = Self;

    fn add(self, other: Vec2i) -> <Self as std::ops::Add<Vec2i>>::Output {
        Vec2i {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rect {
    pub pos: Vec2i,
    pub sz: Vec2i,
}

impl Rect {
    pub fn contains(&self, other: Rect) -> bool {
        let br = self.pos + self.sz;
        let obr = other.pos + other.sz;
        self.pos.x <= other.pos.x && self.pos.y <= other.pos.y && obr.x <= br.x && obr.y <= br.y
    }
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct Image {
    pub buffer: Box<[Color]>, // or Vec<Color>, or...
    pub sz: Vec2i,
}

impl Image {
    // maybe a function to load it from a file using the png crate!
    // you'd want to premultiply it and convert it from `[u8]` into `[Color]`!
    pub fn new(sz: Vec2i) -> Self {
        Self {
            buffer: vec![(0, 0, 0, 255); (sz.x * sz.y) as usize].into_boxed_slice(),
            sz,
        }
    }
    pub fn as_slice(&self) -> &[Color] {
        &self.buffer
    }
    pub fn from_file(p: &std::path::Path) -> Self {
        use std::fs::File;
        let decoder = png::Decoder::new(File::open(p).unwrap());
        let mut reader = decoder.read_info().unwrap();
        // Allocate the output buffer.
        let mut buf = vec![0; reader.output_buffer_size()];
        // Read the next frame. An APNG might contain multiple frames.
        let info = reader.next_frame(&mut buf).unwrap();
        assert_eq!(info.color_type, png::ColorType::Rgba);
        Self {
            buffer: buf
                .chunks_exact(4)
                .map(|px| {
                    // premultiply alpha
                    let a = px[3] as f32 / 255.0;
                    let r = (px[0] as f32 * a) as u8;
                    let g = (px[1] as f32 * a) as u8;
                    let b = (px[2] as f32 * a) as u8;
                    (r, g, b, a as u8) // Color
                })
                .collect::<Box<[Color]>>(),
            sz: Vec2i {
                x: info.width as i32,
                y: info.height as i32,
            },
        }
    }

    pub fn clear(&mut self, c: Color) {
        self.buffer.fill(c);
    }

    pub fn hline(&mut self, x0: usize, x1: usize, y: usize, c: Color) {
        assert!(y < self.sz.y as usize);
        assert!(x0 <= x1);
        assert!(x1 < self.sz.x as usize);
        self.buffer[y * self.sz.x as usize + x0..(y * self.sz.x as usize + x1)].fill(c);
    }

    // maybe put bitblt into here?
    pub fn bitblt(&mut self, src: &Image, from: Rect, to: Vec2i) {
        assert!(Rect {
            pos: Vec2i { x: 0, y: 0 },
            sz: src.sz
        }
        .contains(from));
        let Vec2i { x: to_x, y: to_y } = to;
        if to_x + from.sz.x < 0 || self.sz.x <= to_x || to_y + from.sz.y < 0 || self.sz.y <= to_y {
            return;
        }
        let src_pitch = src.sz.x as usize;
        let dst_pitch = self.sz.x as usize;
        // All this rigmarole is just to avoid bounds checks on each pixel of the blit.
        // We want to calculate which row/col of the src image to start at and which to end at.
        // This way there's no need to even check for out of bounds draws---
        // we'll skip rows that are off the top or off the bottom of the image
        // and skip columns off the left or right sides.
        let y_skip = to_y.max(0) - to_y;
        let x_skip = to_x.max(0) - to_x;
        let y_count = (to_y + from.sz.y as i32).min(self.sz.y) - to_y;
        let x_count = (to_x + from.sz.x as i32).min(self.sz.x) - to_x;
        // The code above is gnarly so these are just for safety:
        debug_assert!(0 <= x_skip);
        debug_assert!(0 <= y_skip);
        debug_assert!(0 <= x_count);
        debug_assert!(0 <= y_count);
        debug_assert!(x_count <= from.sz.x);
        debug_assert!(y_count <= from.sz.y);
        debug_assert!(0 <= to_x + x_skip);
        debug_assert!(0 <= to_y + y_skip);
        debug_assert!(0 <= from.pos.x + x_skip);
        debug_assert!(0 <= from.pos.y + y_skip);
        debug_assert!(to_x + x_count <= self.sz.x);
        debug_assert!(to_y + y_count <= self.sz.y);
        // OK, let's do some copying now
        let from_start: usize = src_pitch * (from.pos.y + y_skip) as usize;
        let from_stop: usize = src_pitch * (from.pos.y + y_count) as usize;
        let to_start: usize = dst_pitch * (to_y + y_skip) as usize;
        let to_stop: usize = dst_pitch * (to_y + y_count) as usize;
        // From the first pixel of the top row to the first pixel of the row past the bottom...
        for (row_a, row_b) in src.buffer[from_start..from_stop]
            // For each whole row...
            .chunks_exact(src_pitch)
            // Tie it up with the corresponding row from dst
            .zip(self.buffer[to_start..to_stop].chunks_exact_mut(dst_pitch))
        {
            // Get column iterators, save on indexing overhead
            let to_row_start = (to_x + x_skip) as usize;
            let to_row_stop = (to_x + x_count) as usize;
            let to_cols = row_b[to_row_start..to_row_stop].iter_mut();
            let from_row_start = (from.pos.x + x_skip) as usize;
            let from_row_stop = (from.pos.x + x_count) as usize;
            let from_cols = row_a[from_row_start..from_row_stop].iter();
            // Composite over, assume premultiplied rgba8888 in src!
            for (to, from) in to_cols.zip(from_cols) {
                let ta = to.3 as f32 / 255.0;
                let fa = from.3 as f32 / 255.0;
                to.0 = from
                    .0
                    .saturating_add((to.0 as f32 * (1.0 - fa)).round() as u8);
                to.1 = from
                    .1
                    .saturating_add((to.1 as f32 * (1.0 - fa)).round() as u8);
                to.2 = from
                    .2
                    .saturating_add((to.2 as f32 * (1.0 - fa)).round() as u8);
                to.3 = ((fa + ta * (1.0 - fa)) * 255.0).round() as u8;
            }
        }
    }

    // ... what else?

    // ... Could our 2d framebuffer itself be an `Image`?
}
