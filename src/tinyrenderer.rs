#![allow(dead_code)]

use std::convert::TryInto;

use image::{Pixel, Rgba, RgbaImage};
use nalgebra::{Point2, Point4, Vector2, Vector4, Matrix4, clamp, Point3};
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
                &Point2::<i32>::new(x0 as i32, y),
                &Point2::<i32>::new(x1 as i32, y),
                color,
                img,
            );
            x1 += invslope20;
            x0 += invslope21;
        }
    } else {
        for y in (base_l.y..=edge.y).rev() {
            draw_line(
                &Point2::<i32>::new(x0 as i32, y),
                &Point2::<i32>::new(x1 as i32, y),
                color,
                img,
            );
            x1 -= invslope20;
            x0 -= invslope21;
        }
    }
}

/// Implementation of line sweeping algorithm for triangle filling
fn draw_face_line_sweeping(screen_coords: &[Point2<i32>; 3], color: Rgba<u8>, img: &mut RgbaImage) {
    let v0 = &screen_coords[0];
    let v1 = &screen_coords[1];
    let v2 = &screen_coords[2];
    let mut points = [v0, v1, v2];
    points.sort_by_key(|k| k.y);
    let [v0, v1, v2] = points;
    let v3_y = v1.y;
    let v3_x =
        v2.x + ((((v1.y - v2.y) as f32) / ((v0.y - v2.y) as f32)) * (v0.x - v2.x) as f32) as i32;
    // v3 corresponds to the point that is at the same height as the second-most vertex height-wise
    // that is at the edge of the triangle that is opposite of such a vertex
    let v3 = Point2::<i32>::new(v3_x, v3_y);
    draw_flat_triangle(&v2, &v1, &v3, color, img);
    draw_flat_triangle(&v0, &v1, &v3, color, img);
}

/// Implementation of barycentric algorithm for triangle filling
fn draw_face_barycentric(
    screen_coords: &[Point2<i32>; 3],
    world_coords: &[Point4<f32>; 3],
    texture_coords: &[Point2<f32>; 3],
    texture: &RgbaImage,
    light_intensity: f32,
    z_buffer: &mut Vec<Vec<f32>>,
    img: &mut RgbaImage,
) {
    let [v0_s, v1_s, v2_s] = &screen_coords;
    let [v0_w, v1_w, v2_w] = &world_coords;
    let [v0_t, v1_t, v2_t] = &texture_coords;
    // Define triangle bounding box
    let max_x = std::cmp::max(v0_s.x, std::cmp::max(v1_s.x, v2_s.x));
    let max_y = std::cmp::max(v0_s.y, std::cmp::max(v1_s.y, v2_s.y));
    let min_x = std::cmp::min(v0_s.x, std::cmp::min(v1_s.x, v2_s.x));
    let min_y = std::cmp::min(v0_s.y, std::cmp::min(v1_s.y, v2_s.y));

    let vec1: Vector2::<i32> = v1_s - v0_s;
    let vec2: Vector2::<i32> = v2_s - v0_s;

    let vec1_x_vec2 = vec1.perp(&vec2) as f32;

    // Calculate if point2 of the bounding box is inside triangle
    for x in min_x..=max_x {
        for y in min_y..max_y {
            let pv0 = Vector2::from(Point2::<i32>::new(x, y) - v0_s);
            let vec1_x_pv0 = vec1.perp(&pv0) as f32;
            let pv0_x_vec2 = pv0.perp(&vec2) as f32;
            // Barycentric coordinates
            let s = vec1_x_pv0 / vec1_x_vec2;
            let t = pv0_x_vec2 / vec1_x_vec2;
            let t_s_1 = 1. - (t + s);

            if s >= 0. && t >= 0. && t_s_1 >= 0. {
                let z_value = t_s_1 * v0_w.z + t * v1_w.z + s * v2_w.z;
                if z_buffer[x as usize][y as usize] < z_value {
                    z_buffer[x as usize][y as usize] = z_value;
                    let tex_x_value = t_s_1 * v0_t.x + t * v1_t.x + s * v2_t.x;
                    let tex_y_value = t_s_1 * v0_t.y + t * v1_t.y + s * v2_t.y;
                    let mut tex_point = texture
                        .get_pixel(tex_x_value as u32, tex_y_value as u32)
                        .to_rgba();
                    tex_point.apply_without_alpha(|ch| ((ch as f32) * light_intensity) as u8);
                    img.put_pixel(x as u32, y as u32, tex_point);
                }
            }
        }
    }
}

fn get_face_screen_coords(
    model: &Obj<TexturedVertex>,
    face: &[u16],
    half_screen_width: f32,
    half_screen_height: f32,
) -> [Point2<i32>; 3] {
    let [v0x, v0y, _] = model.vertices[face[0] as usize].position;
    let [v1x, v1y, _] = model.vertices[face[1] as usize].position;
    let [v2x, v2y, _] = model.vertices[face[2] as usize].position;
    let point0 = Point2::<i32>::new(
        ((v0x + 1.) * half_screen_width) as i32,
        ((v0y + 1.) * half_screen_height) as i32,
    );
    let point1 = Point2::<i32>::new(
        ((v1x + 1.) * half_screen_width) as i32,
        ((v1y + 1.) * half_screen_height) as i32,
    );
    let point2 = Point2::<i32>::new(
        ((v2x + 1.) * half_screen_width) as i32,
        ((v2y + 1.) * half_screen_height) as i32,
    );
    [point0, point1, point2]
}

fn world_to_screen_coords(
    world_coords: Point4<f32>,
    half_screen_width: f32,
    half_screen_height: f32,
) -> Point2<i32> {
    Point2::<i32>::new(
        ((world_coords.x + 1.) * half_screen_width) as i32,
        ((world_coords.y + 1.) * half_screen_height) as i32,
    )
}

fn get_face_world_coords(model: &Obj<TexturedVertex>, face: &[u16]) -> [Point4<f32>; 3] {
    let [v0x, v0y, v0z] = model.vertices[face[0] as usize].position;
    let [v1x, v1y, v1z] = model.vertices[face[1] as usize].position;
    let [v2x, v2y, v2z] = model.vertices[face[2] as usize].position;
    let point0 = Point4::<f32>::new(v0x, v0y, v0z, 1.);
    let point1 = Point4::<f32>::new(v1x, v1y, v1z, 1.);
    let point2 = Point4::<f32>::new(v2x, v2y, v2z, 1.);
    [point0, point1, point2]
}

fn get_face_texture_coords(
    model: &Obj<TexturedVertex>,
    face: &[u16],
    texture_width: f32,
    texture_height: f32,
) -> [Point2<f32>; 3] {
    let [v0x, v0y, _] = model.vertices[face[0] as usize].texture;
    let [v1x, v1y, _] = model.vertices[face[1] as usize].texture;
    let [v2x, v2y, _] = model.vertices[face[2] as usize].texture;
    let texcoord0 = Point2::<f32>::new((v0x * texture_width) - 1., (v0y * texture_height) - 1.);
    let texcoord1 = Point2::<f32>::new((v1x * texture_width) - 1., (v1y * texture_height) - 1.);
    let texcoord2 = Point2::<f32>::new((v2x * texture_width) - 1., (v2y * texture_height) - 1.);
    [texcoord0, texcoord1, texcoord2]
}

fn calc_light_intensity(world_coords: &[Point4<f32>; 3], light_dir: Vector4<f32>) -> f32 {
    let vec0: Vector4<f32> = world_coords[0] - world_coords[1];
    let vec1: Vector4<f32> = world_coords[0] - world_coords[2];
    let norm = vec1.xyz().cross(&vec0.xyz()).normalize();
    norm.dot(&light_dir.xyz())
}

/// Draw triangle faces of given 3D object
pub fn draw_faces(model: Obj<TexturedVertex>, img: &mut RgbaImage, texture: RgbaImage) {
    let faces_num = model.indices.len();
    let faces = &model.indices[..faces_num];
    let (half_screen_width, half_screen_height) = (
        ((img.width() - 1) / 2) as f32,
        ((img.height() - 1) / 2) as f32,
    );

    let mut z_buffer = vec![vec![f32::NEG_INFINITY; img.height() as usize]; img.width() as usize];

    let camera = Point3::new(0., 0., 3.);
    let camera = Matrix4::<f32>::new(1., 0. ,0., 0.,
                                     0., 1., 0., 0.,
                                     0., 0., 1., 0.,
                                     0., 0., -1./camera.z, 1.);

    for face in faces.chunks(3) {
        let mut world_coords = get_face_world_coords(&model, face);
        for coord in world_coords.iter_mut() {
            // println!("Pre {}", coord);
            *coord = Point4::from(camera * coord.coords);
            *coord /= coord.w;
            // println!("Post {}", coord);
            coord.x = clamp(coord.x, -1.0, 1.0);
            coord.y = clamp(coord.y, -1.0, 1.0);
            coord.z = clamp(coord.z, -1.0, 1.0);
        }
        let screen_coords: [Point2<i32>; 3] = world_coords
            .iter()
            .map(|coord| world_to_screen_coords(*coord, half_screen_width, half_screen_height))
            .collect::<Vec<Point2<i32>>>()
            .try_into()
            .unwrap();
        let texture_coords = get_face_texture_coords(
            &model,
            face,
            texture.width() as f32,
            texture.height() as f32,
        );

        let light_dir = Vector4::new(0., 0., -1., 0.);
        let light_intensity = calc_light_intensity(&world_coords, light_dir);
        // Draw face
        if light_intensity > 0. {
            draw_face_barycentric(
                &screen_coords,
                &world_coords,
                &texture_coords,
                &texture,
                light_intensity,
                &mut z_buffer,
                img,
            );
        }
    }
}
