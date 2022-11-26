#![allow(dead_code)]

use image::{Rgba, RgbaImage};
use obj::Obj;
use rand::Rng;
use tinyrenderer::structs::{Point2, Vec2};

use self::structs::{Point3, Vec3};

/// Implementation of the Bresenham's line algorithm
/// Returns a vector of points with each point representing the coordinates of the pixels to be
/// drawn.
fn line(start: &Point2<i32>, end: &Point2<i32>) -> Vec<Point2<i32>> {
    let Point2::<i32> {
        x: mut x0,
        y: mut y0,
    } = start;
    let Point2::<i32> {
        x: mut x1,
        y: mut y1,
    } = end;

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
            result.push(Point2::<i32> { x: y, y: x });
            if d > 0 {
                y = y + yi;
                d = d - 2 * dx;
            }
            d = d + 2 * dy;
        }
    } else {
        for x in x0..=x1 {
            result.push(Point2::<i32> { x, y });
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
    while let Some(Point2::<i32> { x, y }) = line_iter.next() {
        img.put_pixel(*x as u32, *y as u32, color);
    }
}

/// Draw mesh's wireframe
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
        let point1 = Point2::<i32> {
            x: ((v1x + 1.) * width_half) as i32,
            y: ((v1y + 1.) * height_half) as i32,
        };
        let point2 = Point2::<i32> {
            x: ((v2x + 1.) * width_half) as i32,
            y: ((v2y + 1.) * height_half) as i32,
        };
        let point3 = Point2::<i32> {
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
    edge: &Point2<i32>,
    base_l: &Point2<i32>,
    base_r: &Point2<i32>,
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
                &Point2::<i32> { x: x0 as i32, y },
                &Point2::<i32> { x: x1 as i32, y },
                color,
                img,
            );
            x1 += invslope20;
            x0 += invslope21;
        }
    } else {
        for y in (base_l.y..=edge.y).rev() {
            draw_line(
                &Point2::<i32> { x: x0 as i32, y },
                &Point2::<i32> { x: x1 as i32, y },
                color,
                img,
            );
            x1 -= invslope20;
            x0 -= invslope21;
        }
    }
}

/// Implementation of line sweeping algorithm for triangle filling
fn draw_face_line_sweeping(
    v0: &Point2<i32>,
    v1: &Point2<i32>,
    v2: &Point2<i32>,
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
    let v3 = Point2::<i32> { x: v3_x, y: v3_y };
    draw_flat_triangle(&v2, &v1, &v3, color, img);
    draw_flat_triangle(&v0, &v1, &v3, color, img);
}

/// Implementation of barycentric algorithm for triangle filling
fn draw_face_barycentric(
    v0: &Point2<i32>,
    v1: &Point2<i32>,
    v2: &Point2<i32>,
    color: Rgba<u8>,
    img: &mut RgbaImage,
) {
    // Define triangle bounding box
    let max_x = std::cmp::max(v0.x, std::cmp::max(v1.x, v2.x));
    let max_y = std::cmp::max(v0.y, std::cmp::max(v1.y, v2.y));
    let min_x = std::cmp::min(v0.x, std::cmp::min(v1.x, v2.x));
    let min_y = std::cmp::min(v0.y, std::cmp::min(v1.y, v2.y));

    let vec1 = Vec2::<i32>::from_points(&v0, &v1);
    let vec2 = Vec2::<i32>::from_points(&v0, &v2);

    let vec1_x_vec2 = Vec2::<i32>::cross(&vec1, &vec2) as f32;

    // Calculate if point2 of the bounding box is inside triangle
    for x in min_x..=max_x {
        for y in min_y..max_y {
            let pv0 = Vec2::from_points(&v0, &Point2::<i32> { x, y });
            let vec1_x_pv0 = Vec2::<i32>::cross(&vec1, &pv0) as f32;
            let pv0_x_vec2 = Vec2::<i32>::cross(&pv0, &vec2) as f32;

            let s = vec1_x_pv0 / vec1_x_vec2;
            let t = pv0_x_vec2 / vec1_x_vec2;

            if s >= 0. && t >= 0. && s + t <= 1. {
                img.put_pixel(x as u32, y as u32, color);
            }
        }
    }
}


fn get_face_screen_coords(
    model: &Obj,
    face: &[u16],
    half_screen_width: f32,
    half_screen_height: f32,
) -> [Point2<i32>; 3] {
    let [v1x, v1y, _] = model.vertices[face[0] as usize].position;
    let [v2x, v2y, _] = model.vertices[face[1] as usize].position;
    let [v3x, v3y, _] = model.vertices[face[2] as usize].position;
    let point1 = Point2::<i32> {
        x: ((v1x + 1.) * half_screen_width) as i32,
        y: ((v1y + 1.) * half_screen_height) as i32,
    };
    let point2 = Point2::<i32> {
        x: ((v2x + 1.) * half_screen_width) as i32,
        y: ((v2y + 1.) * half_screen_height) as i32,
    };
    let point3 = Point2::<i32> {
        x: ((v3x + 1.) * half_screen_width) as i32,
        y: ((v3y + 1.) * half_screen_height) as i32,
    };
    [point1, point2, point3]
}

fn get_face_world_coords(model: &Obj, face: &[u16]) -> [Point3<f32>; 3] {
    let [v1x, v1y, v1z] = model.vertices[face[0] as usize].position;
    let [v2x, v2y, v2z] = model.vertices[face[1] as usize].position;
    let [v3x, v3y, v3z] = model.vertices[face[2] as usize].position;
    let point1 = Point3::<f32> {
        x: v1x,
        y: v1y,
        z: v1z,
    };
    let point2 = Point3::<f32> {
        x: v2x,
        y: v2y,
        z: v2z,
    };
    let point3 = Point3::<f32> {
        x: v3x,
        y: v3y,
        z: v3z,
    };
    [point1, point2, point3]
}

/// Draw triangle faces of given 3D object
pub fn draw_faces(model: Obj, img: &mut RgbaImage) {
    let faces_num = model.indices.len();
    let faces = &model.indices[..faces_num];
    let (width_half, height_half) = (
        ((img.width() - 1) / 2) as f32,
        ((img.height() - 1) / 2) as f32,
    );

    for face in faces.chunks(3) {
        let screen_coords = get_face_screen_coords(&model, face, width_half, height_half);
        // Draw face
        let mut rng = rand::thread_rng();
        let color = Rgba([rng.gen(), rng.gen(), rng.gen(), 255]);
        draw_face_barycentric(&screen_coords[0], &screen_coords[1], &screen_coords[2], color, img);
    }
}

mod structs;
