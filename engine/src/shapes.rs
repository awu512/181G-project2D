type Color = (u8, u8, u8, u8);

const WIDTH: usize = 320;
const HEIGHT: usize = 240;

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32, // Float positions and extents could also be fine
}

impl Rect {
    pub fn new(x: i32, y: i32, w: u32, h: u32) -> Rect {
        Rect { x, y, w, h }
    }
    // Maybe add functions to test if a point is inside the rect...
    // Or whether two rects overlap...
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub struct Vec2i {
    // Or Vec2f for floats?
    pub x: i32,
    pub y: i32,
}

impl Vec2i {
    pub fn new(x: i32, y: i32) -> Vec2i {
        Vec2i { x, y }
    }
    // Maybe add functions for e.g. the midpoint of two vecs, or...
}

// Maybe add implementations of traits like std::ops::Add, AddAssign, Mul, MulAssign, Sub, SubAssign, ...

// Here's what clear looks like, though we won't use it
#[allow(dead_code)]
pub fn clear(fb: &mut [Color], c: Color) {
    fb.fill(c);
}

#[allow(dead_code)]
pub fn hline(fb: &mut [Color], x0: usize, x1: usize, y: usize, c: Color) {
    assert!(y < HEIGHT);
    assert!(x0 <= x1);
    assert!(x1 < WIDTH);
    fb[y * WIDTH + x0..(y * WIDTH + x1)].fill(c);
}

#[allow(dead_code)]
pub fn vline(fb: &mut [Color], x: usize, y0: usize, y1: usize, c: Color) {
    assert!(y0 <= y1);
    assert!(y1 <= HEIGHT);
    assert!(x < WIDTH);
    let rect_height = y1 - y0;
    let mut x_level = x;
    for _ in 0..rect_height + 1 {
        fb[y0 * WIDTH + x_level..(y0 * WIDTH + x_level + 1)].fill(c);
        x_level = x_level + WIDTH;
    }
}

#[allow(dead_code)]
pub fn draw_filled_rect(fb2d: &mut [Color], starting_height: usize, w: usize, color: (u8, u8, u8, u8)) {
    if starting_height + 15 < HEIGHT {
        for y in starting_height..starting_height + 15 {
            hline(fb2d, WIDTH / 2 - w / 2, WIDTH / 2 + w / 2, y, color);
        }
    }
}

#[allow(dead_code)]
pub fn draw_outlined_rect(
    fb2d: &mut [Color],
    starting_height: usize,
    w: usize,
    color: (u8, u8, u8, u8),
) {
    if starting_height + 15 < HEIGHT {
        // Top of rect
        hline(
            fb2d,
            WIDTH / 2 - w / 2,
            WIDTH / 2 + w / 2,
            starting_height,
            color,
        );
        // Left side of rect
        vline(
            fb2d,
            WIDTH / 2 - w / 2,
            starting_height,
            starting_height + 15,
            color,
        );
        // // Right side of rect
        vline(
            fb2d,
            WIDTH / 2 + w / 2,
            starting_height,
            starting_height + 15,
            color,
        );
        // Bottom of rect
        hline(
            fb2d,
            WIDTH / 2 - w / 2,
            WIDTH / 2 + w / 2,
            starting_height + 15,
            color,
        );
    }
}

#[allow(dead_code)]
pub fn diagonal_line(fb: &mut [Color], (x0, y0): (usize, usize), (x1, y1): (usize, usize), col: Color) {
    let mut x = x0 as i64;
    let mut y = y0 as i64;
    let x0 = x0 as i64;
    let y0 = y0 as i64;
    let x1 = x1 as i64;
    let y1 = y1 as i64;
    let dx = (x1 - x0).abs();
    let sx: i64 = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy: i64 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    while x != x1 || y != y1 {
        fb[(y as usize * WIDTH + x as usize)..(y as usize * WIDTH + (x as usize + 1))].fill(col);
        let e2 = 2 * err;
        if dy <= e2 {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

#[allow(dead_code)]
fn hline_beyond_window(fb: &mut [Color], x0: usize, x1: usize, y: usize, c: Color) {
    fb[y * WIDTH + x0..(y * WIDTH + x1)].fill(c);
}