pub type Color = (u8, u8, u8, u8);

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub struct Vec2i {
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

    pub fn contains_point(&self, point: Vec2i) -> bool {
        self.pos.x <= point.x && 
        self.pos.y <= point.y && 
        self.pos.x + self.sz.x >= point.x && 
        self.pos.y + self.sz.y >= point.y
    }

    pub fn move_by(&mut self, dx: i32, dy: i32) {
        self.pos.x += dx;
        self.pos.y += dy;
    }

    pub fn bottom(&self) -> i32 {
        self.pos.y + self.sz.y
    }
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct Image {
    pub buffer: Box<[Color]>,
    pub sz: Vec2i,
}

impl Image {
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
        let img = image_reading::open(p).unwrap().into_rgba8();
        let sz = Vec2i {
            x: img.width() as i32,
            y: img.height() as i32,
        };
        let img = img.into_vec();
        Self {
            buffer: img
                .chunks_exact(4)
                .map(|px| {
                    let a = px[3] as f32 / 255.0;
                    let r = (px[0] as f32 * a) as u8;
                    let g = (px[1] as f32 * a) as u8;
                    let b = (px[2] as f32 * a) as u8;
                    (r, g, b, (a * 255.0) as u8)
                })
                .collect(),
            sz,
        }
    }

    pub fn size(&self) -> (i32, i32) {
        (self.sz.x, self.sz.y)
    }

    pub fn clear(&mut self, c: Color) {
        self.buffer.fill(c);
    }

    pub fn draw_rect(&mut self, rect: &Rect, color: Color) {
        for y in (rect.pos.y)..(rect.pos.y + rect.sz.y) {
            for x in (rect.pos.x)..(rect.pos.x + rect.sz.x) {
                if (y*self.sz.x + x) < self.sz.x * self.sz.y && (y*self.sz.x + x) >= 0 {
                    self.buffer[(y*self.sz.x + x) as usize..((y*self.sz.x + x) as usize) + 1]
                        .fill(color);
                }
            }
        }
    }

    pub fn draw_ball(&mut self, rect: &Rect, (r,g,_b,_a): Color) {
        let ball = vec![
            0, 0, 1, 1, 1, 1, 0, 0,
            0, 1, 1, 1, 1, 1, 1, 0,
            1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1,
            0, 1, 1, 1, 1, 1, 1, 0,
            0, 0, 1, 1, 1, 1, 0, 0
        ];
        let mut coli: usize = 0;

        for y in (rect.pos.y)..(rect.pos.y + rect.sz.y) {
            for x in (rect.pos.x)..(rect.pos.x + rect.sz.x) {
                if (y*self.sz.x + x) < self.sz.x * self.sz.y 
                    && x >= 0 
                    && ball[coli] > 0 {
                        let col: Color = (r*ball[coli],g*ball[coli],0,255);
                        self.buffer[(y*self.sz.x + x) as usize..((y*self.sz.x + x) as usize) + 1]
                            .fill(col);
                }
                coli += 1;
            }
        }
    }

    pub fn hline(&mut self, x0: usize, x1: usize, y: usize, c: Color) {
        assert!(y < self.sz.y as usize);
        assert!(x0 <= x1);
        assert!(x1 < self.sz.x as usize);
        self.buffer[y * self.sz.x as usize + x0..(y * self.sz.x as usize + x1)].fill(c);
    }

    pub fn bitblt(&mut self, src: &Image, from: Rect, to: Vec2i, flip: bool) {
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

        let y_skip = to_y.max(0) - to_y;
        let x_skip = to_x.max(0) - to_x;
        let y_count = (to_y + from.sz.y as i32).min(self.sz.y) - to_y;
        let x_count = (to_x + from.sz.x as i32).min(self.sz.x) - to_x;
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
        let from_start: usize = src_pitch * (from.pos.y + y_skip) as usize;
        let from_stop: usize = src_pitch * (from.pos.y + y_count) as usize;
        let to_start: usize = dst_pitch * (to_y + y_skip) as usize;
        let to_stop: usize = dst_pitch * (to_y + y_count) as usize;
        for (row_a, row_b) in src.buffer[from_start..from_stop]
            .chunks_exact(src_pitch)
            .zip(self.buffer[to_start..to_stop].chunks_exact_mut(dst_pitch))
        {
            let to_row_start = (to_x + x_skip) as usize;
            let to_row_stop = (to_x + x_count) as usize;
            let to_cols = row_b[to_row_start..to_row_stop].iter_mut();
            let from_row_start = (from.pos.x + x_skip) as usize;
            let from_row_stop = (from.pos.x + x_count) as usize;
            let from_cols = row_a[from_row_start..from_row_stop].iter();
            let from_cols = if flip {
                Box::new(row_a[from_row_start..from_row_stop].iter().rev())
                    as Box<dyn Iterator<Item = &Color>>
            } else {
                Box::new(from_cols) as Box<dyn Iterator<Item = &Color>>
            };
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
}


