use image::{Rgba, RgbaImage};
use nalgebra::Point2;

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
                y = y + yi;
                d = d - 2 * dx;
            }
            d = d + 2 * dy;
        }
    } else {
        for x in x0..=x1 {
            result.push(Point2::<i32>::new(x, y));
            if d > 0 {
                y = y + yi;
                d = d - 2 * dx;
            }
            d = d + 2 * dy;
        }
    };
    result
}

/// Draw line given a set of points
pub fn draw_line(start: &Point2<i32>, end: &Point2<i32>, color: Rgba<u8>, img: &mut RgbaImage) {
    let line = line(&start, &end);
    let mut line_iter = line.iter();
    while let Some(point) = line_iter.next() {
        img.put_pixel(point.x as u32, point.y as u32, color);
    }
}
