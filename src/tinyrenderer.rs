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

pub fn draw_wireframe(model: Obj, color: Rgba<u8>, img: &mut RgbaImage) {
    let faces_num = model.indices.len();
    let faces = &model.indices[..faces_num];
    let (width_half, height_half) = (
        ((img.width() - 1) / 2) as f32,
        (((img.height() - 1) / 2) as f32),
    );
    for face in faces.chunks(3) {
        let [v1x, v1y, _] = model.vertices[face[0] as usize].position;
        let [v2x, v2y, _] = model.vertices[face[1] as usize].position;
        let [v3x, v3y, _] = model.vertices[face[2] as usize].position;
        let (v1x, v1y) = (
            ((v1x + 1.) * width_half) as i32,
            ((v1y + 1.) * height_half) as i32,
        );
        let (v2x, v2y) = (
            ((v2x + 1.) * width_half) as i32,
            ((v2y + 1.) * height_half) as i32,
        );
        let (v3x, v3y) = (
            ((v3x + 1.) * width_half) as i32,
            ((v3y + 1.) * height_half) as i32,
        );
        println!("{}, {}", v1x, v1y);
        println!("{}, {}", v2x, v2y);
        println!("{}, {}", v3x, v3y);
        // Draw triangle
        draw_line(
            Point {
                x: v1x as i32,
                y: v1y as i32,
            },
            Point {
                x: v2x as i32,
                y: v2y as i32,
            },
            color,
            img,
        );
        draw_line(
            Point {
                x: v2x as i32,
                y: v2y as i32,
            },
            Point {
                x: v3x as i32,
                y: v3y as i32,
            },
            color,
            img,
        );
        draw_line(
            Point {
                x: v3x as i32,
                y: v3y as i32,
            },
            Point {
                x: v1x as i32,
                y: v1y as i32,
            },
            color,
            img,
        );
    }
}
