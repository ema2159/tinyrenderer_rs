use image::{Rgba, RgbaImage};
use nalgebra::Point2;
use obj::{Obj, TexturedVertex};

/// Implementation of the Bresenham's line algorithm
/// Returns a vector of points with each point representing the coordinates of the pixels to be
/// drawn.
fn line(start: &Point2<i32>, end: &Point2<i32>) -> Vec<Point2<i32>> {
    let (mut x0, mut y0) = (start.x, start.y);
    let (mut x1, mut y1) = (end.x, end.y);

    let mut result = Vec::<Point2<i32>>::new();

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
            result.push(Point2::<i32>::new(y, x));
            if d > 0 {
                y += yi;
                d -= 2 * dx;
            }
            d += 2 * dy;
        }
    } else {
        for x in x0..=x1 {
            result.push(Point2::<i32>::new(x, y));
            if d > 0 {
                y += yi;
                d -= 2 * dx;
            }
            d += 2 * dy;
        }
    };
    result
}

/// Draw line given a set of points
pub fn draw_line(start: &Point2<i32>, end: &Point2<i32>, color: Rgba<u8>, img: &mut RgbaImage) {
    let line = line(start, end);
    let line_iter = line.iter();
    for point in line_iter {
        img.put_pixel(point.x as u32, point.y as u32, color);
    }
}

/// Draw mesh's wireframe
pub fn draw_wireframe(model: Obj<TexturedVertex>, color: Rgba<u8>, img: &mut RgbaImage) {
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
        let point1 = Point2::<i32>::new(
            ((v1x + 1.) * width_half) as i32,
            ((v1y + 1.) * height_half) as i32,
        );
        let point2 = Point2::<i32>::new(
            ((v2x + 1.) * width_half) as i32,
            ((v2y + 1.) * height_half) as i32,
        );
        let point3 = Point2::<i32>::new(
            ((v3x + 1.) * width_half) as i32,
            ((v3y + 1.) * height_half) as i32,
        );
        // Draw triangle
        draw_line(&point1, &point2, color, img);
        draw_line(&point2, &point3, color, img);
        draw_line(&point3, &point1, color, img);
    }
}
