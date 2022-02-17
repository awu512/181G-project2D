type Color = (u8, u8, u8, u8);

const WIDTH: usize = 320;
const HEIGHT: usize = 240;

#[derive(Clone, Copy, Hash, Debug)]
pub struct Fb2d {
    pub color: Color,
    pub array: [Color; HEIGHT * WIDTH],
}

impl Fb2d {
    pub fn new(color: Color) -> Fb2d {
        let array = [color; HEIGHT * WIDTH];
        Fb2d { color, array }
    }

    // Here's what clear looks like, though we won't use it
    #[allow(dead_code)]
    pub fn clear(&mut self, c: Color) {
        self.color = c;
        self.array.fill(c);
    }

    #[allow(dead_code)]
    pub fn hline(&mut self, x0: usize, x1: usize, y: usize, c: Color) {
        assert!(y < HEIGHT);
        assert!(x0 <= x1);
        assert!(x1 < WIDTH);
        self.array[y * WIDTH + x0..(y * WIDTH + x1)].fill(c);
    }

    #[allow(dead_code)]
    pub fn vline(&mut self, x: usize, y0: usize, y1: usize, c: Color) {
        assert!(y0 <= y1);
        assert!(y1 <= HEIGHT);
        assert!(x < WIDTH);
        let rect_height = y1 - y0;
        let mut x_level = x;
        for _ in 0..rect_height + 1 {
            self.array[y0 * WIDTH + x_level..(y0 * WIDTH + x_level + 1)].fill(c);
            x_level = x_level + WIDTH;
        }
    }

    #[allow(dead_code)]
    pub fn draw_filled_rect(&mut self, starting_height: usize, w: usize, color: Color) {
        if starting_height + 15 < HEIGHT {
            for y in starting_height..starting_height + 15 {
                self.hline(WIDTH / 2 - w / 2, WIDTH / 2 + w / 2, y, color);
            }
        }
    }

    #[allow(dead_code)]
    pub fn draw_outlined_rect(&mut self, starting_height: usize, w: usize, color: Color) {
        if starting_height + 15 < HEIGHT {
            // Top of rect
            self.hline(WIDTH / 2 - w / 2, WIDTH / 2 + w / 2, starting_height, color);
            // Left side of rect
            self.vline(
                WIDTH / 2 - w / 2,
                starting_height,
                starting_height + 15,
                color,
            );
            // // Right side of rect
            self.vline(
                WIDTH / 2 + w / 2,
                starting_height,
                starting_height + 15,
                color,
            );
            // Bottom of rect
            self.hline(
                WIDTH / 2 - w / 2,
                WIDTH / 2 + w / 2,
                starting_height + 15,
                color,
            );
        }
    }

    #[allow(dead_code)]
    pub fn diagonal_line(
        &mut self,
        (x0, y0): (usize, usize),
        (x1, y1): (usize, usize),
        col: Color,
    ) {
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
            self.array[(y as usize * WIDTH + x as usize)..(y as usize * WIDTH + (x as usize + 1))]
                .fill(col);
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
}