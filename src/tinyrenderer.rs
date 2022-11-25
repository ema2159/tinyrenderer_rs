#![allow(dead_code)]
use image::{Rgba, RgbaImage};
use obj::Obj;

#[derive(Debug)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2<T>
where
    T: std::ops::Sub<Output = T> + Copy,
{
    fn from_points(point1: &Point<T>, point2: &Point<T>) -> Vec2<T> {
        Vec2::<T> {
            x: point2.x - point1.x,
            y: point2.y - point1.y,
        }
    }
}

/// Implementation of the Bresenham's line algorithm
/// Returns a vector of points with each point representing the coordinates of the pixels to be
/// drawn.
fn line(start: &Point<i32>, end: &Point<i32>) -> Vec<Point<i32>> {
    let Point::<i32> {
        x: mut x0,
        y: mut y0,
    } = start;
    let Point::<i32> {
        x: mut x1,
        y: mut y1,
    } = end;

    let mut result = Vec::<Point<i32>>::new();

    // Consider case in which slope is more than 1
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
            result.push(Point::<i32> { x: y, y: x });
            if d > 0 {
                y = y + yi;
                d = d - 2 * dx;
            }
            d = d + 2 * dy;
        }
    } else {
        for x in x0..=x1 {
            result.push(Point::<i32> { x, y });
            if d > 0 {
                y = y + yi;
                d = d - 2 * dx;
            }
            d = d + 2 * dy;
        }
    };
    result
}

// Draw line calculated with Bresenham's algorithm
pub fn draw_line(start: &Point<i32>, end: &Point<i32>, color: Rgba<u8>, img: &mut RgbaImage) {
    let line = line(&start, &end);
    let mut line_iter = line.iter();
    while let Some(Point::<i32> { x, y }) = line_iter.next() {
        img.put_pixel(*x as u32, *y as u32, color);
    }
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
        let point1 = Point::<i32> {
            x: ((v1x + 1.) * width_half) as i32,
            y: ((v1y + 1.) * height_half) as i32,
        };
        let point2 = Point::<i32> {
            x: ((v2x + 1.) * width_half) as i32,
            y: ((v2y + 1.) * height_half) as i32,
        };
        let point3 = Point::<i32> {
            x: ((v3x + 1.) * width_half) as i32,
            y: ((v3y + 1.) * height_half) as i32,
        };
        // Draw triangle
        draw_line(&point1, &point2, color, img);
        draw_line(&point2, &point3, color, img);
        draw_line(&point3, &point1, color, img);
    }
}

fn draw_flat_triangle(
    edge: &Point<i32>,
    base_l: &Point<i32>,
    base_r: &Point<i32>,
    color: Rgba<u8>,
    img: &mut RgbaImage,
) {
    let invslope20: f32 = (base_l.x - edge.x) as f32 / (base_l.y - edge.y) as f32;
    let invslope21: f32 = (base_r.x - edge.x) as f32 / (base_r.y - edge.y) as f32;
    let mut x0 = edge.x as f32;
    let mut x1 = edge.x as f32;
    if edge.y < base_l.y {
        // =base_l.y = base_r.y
        for y in edge.y..=base_l.y {
            draw_line(
                &Point::<i32> { x: x0 as i32, y },
                &Point::<i32> { x: x1 as i32, y },
                color,
                img,
            );
            x1 += invslope20;
            x0 += invslope21;
        }
    } else {
        for y in (base_l.y..=edge.y).rev() {
            draw_line(
                &Point::<i32> { x: x0 as i32, y },
                &Point::<i32> { x: x1 as i32, y },
                color,
                img,
            );
            x1 -= invslope20;
            x0 -= invslope21;
        }
    }
}

fn draw_face_line_sweeping(
    v0: Point<i32>,
    v1: Point<i32>,
    v2: Point<i32>,
    color: Rgba<u8>,
    img: &mut RgbaImage,
) {
    let mut points = [v0, v1, v2];
    points.sort_by_key(|k| k.y);
    let [v0, v1, v2] = points;
    let v3_y = v1.y;
    let v3_x =
        v2.x + ((((v1.y - v2.y) as f32) / ((v0.y - v2.y) as f32)) * (v0.x - v2.x) as f32) as i32;
    // v3 corresponds to the point that is at the same height as the second-most vertex height-wise
    // that is at the edge of the triangle that is opposite of such a vertex
    let v3 = Point::<i32> { x: v3_x, y: v3_y };
    draw_flat_triangle(&v2, &v1, &v3, color, img);
    draw_flat_triangle(&v0, &v1, &v3, color, img);
}

pub fn draw_faces(model: Obj, color: Rgba<u8>, img: &mut RgbaImage) {
    let faces_num = model.indices.len();
    let faces = &model.indices[..faces_num];
    let (width_half, height_half) = (
        ((img.width() - 1) / 2) as f32,
        ((img.height() - 1) / 2) as f32,
    );

    for face in faces.chunks(3) {
        let [v1x, v1y, _] = model.vertices[face[0] as usize].position;
        let [v2x, v2y, _] = model.vertices[face[1] as usize].position;
        let [v3x, v3y, _] = model.vertices[face[2] as usize].position;
        let point1 = Point::<i32> {
            x: ((v1x + 1.) * width_half) as i32,
            y: ((v1y + 1.) * height_half) as i32,
        };
        let point2 = Point::<i32> {
            x: ((v2x + 1.) * width_half) as i32,
            y: ((v2y + 1.) * height_half) as i32,
        };
        let point3 = Point::<i32> {
            x: ((v3x + 1.) * width_half) as i32,
            y: ((v3y + 1.) * height_half) as i32,
        };
        // Draw face
        draw_face_line_sweeping(point1, point2, point3, color, img);
    }
}
