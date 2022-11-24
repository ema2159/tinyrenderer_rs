use image::{Rgba, RgbaImage};
use obj::Obj;

pub struct Point {
    pub x: i32,
    pub y: i32,
}

// Implementation of the Bresenham's line algorithm
pub fn draw_line(start: Point, end: Point, color: Rgba<u8>, img: &mut RgbaImage) {
    let Point {
        x: mut x0,
        y: mut y0,
    } = start;
    let Point {
        x: mut x1,
        y: mut y1,
    } = end;

    let mut steep = false;
    if (x1 - x0).abs() < (y1 - y0).abs() {
        (x0, y0) = (y0, x0);
        (x1, y1) = (y1, x1);
        steep = true;
    }

    // Always draw from lowest x to highest x
    if x1 < x0 {
        (x1, x0) = (x0, x1);
        (y1, y0) = (y0, y1);
    }
    let dx = x1 - x0;
    let mut dy = y1 - y0;
    let mut yi = 1;

    // Allow negative slopes
    if dy < 0 {
        (yi, dy) = (-1, -dy)
    };

    let mut d = 2 * dy - dx;
    let mut y = y0;
    if steep {
        for x in x0..=x1 {
            img.put_pixel(y as u32, x as u32, color);
            if d > 0 {
                y = y + yi;
                d = d - 2 * dx;
            }
            d = d + 2 * dy;
        }
    } else {
        for x in x0..=x1 {
            img.put_pixel(x as u32, y as u32, color);
            if d > 0 {
                y = y + yi;
                d = d - 2 * dx;
            }
            d = d + 2 * dy;
        }
    };
}
