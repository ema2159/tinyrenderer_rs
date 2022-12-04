#![allow(dead_code)]

use image::{Pixel, Rgba, RgbaImage};
use nalgebra::{clamp, Matrix4, Point2, Point3, Point4, RowVector4, Vector2, Vector3};
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
    world_coords: &[Point4<f32>; 3],
    texture_coords: &[Point2<f32>; 3],
    texture: &RgbaImage,
    normal_coords: &[Vector3<f32>; 3],
    light: Vector3<f32>,
    z_buffer: &mut Vec<Vec<f32>>,
    img: &mut RgbaImage,
) {
    let [v0_w, v1_w, v2_w] = &world_coords;
    let [v0_t, v1_t, v2_t] = &texture_coords;
    let [v0_n, v1_n, v2_n] = &normal_coords;
    // Define triangle bounding box
    let max_x = (f32::max(v0_w.x, f32::max(v1_w.x, v2_w.x)) + 1.) as i32;
    let max_y = (f32::max(v0_w.y, f32::max(v1_w.y, v2_w.y)) + 1.) as i32;
    let min_x = (f32::min(v0_w.x, f32::min(v1_w.x, v2_w.x)) + 1.) as i32;
    let min_y = (f32::min(v0_w.y, f32::min(v1_w.y, v2_w.y)) + 1.) as i32;

    let vec1: Vector2<f32> = (v1_w - v0_w).xy();
    let vec2: Vector2<f32> = (v2_w - v0_w).xy();

    let vec1_x_vec2 = vec1.perp(&vec2) as f32;

    // Calculate if point2 of the bounding box is inside triangle
    for x in min_x..=max_x {
        for y in min_y..max_y {
            let pv0 = Vector2::from(Point2::<f32>::new(x as f32, y as f32) - v0_w.xy());
            let vec1_x_pv0 = vec1.perp(&pv0) as f32;
            let pv0_x_vec2 = pv0.perp(&vec2) as f32;
            // Barycentric coordinates
            let s = vec1_x_pv0 / vec1_x_vec2;
            let t = pv0_x_vec2 / vec1_x_vec2;
            let t_s_1 = 1. - (t + s);

            if s >= 0. && t >= 0. && t_s_1 >= 0. {
                let z_value = t_s_1 * v0_w.z + t * v1_w.z + s * v2_w.z;
                let normal = t_s_1 * v0_n + t * v1_n + s * v2_n;
                let light_intensity = normal.dot(&light);
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

fn get_face_normal_coords(model: &Obj<TexturedVertex>, face: &[u16]) -> [Vector3<f32>; 3] {
    let [v0x, v0y, v0z] = model.vertices[face[0] as usize].normal;
    let [v1x, v1y, v1z] = model.vertices[face[1] as usize].normal;
    let [v2x, v2y, v2z] = model.vertices[face[2] as usize].normal;
    let normal0 = Vector3::<f32>::new(v0x, v0y, v0z);
    let normal1 = Vector3::<f32>::new(v1x, v1y, v1z);
    let normal2 = Vector3::<f32>::new(v2x, v2y, v2z);
    [normal0, normal1, normal2]
}

fn get_model_view_matrix(
    eye_pos: Point3<f32>,
    model_pos: Point3<f32>,
    up_vector: Vector3<f32>,
) -> Matrix4<f32> {
    let new_z = (eye_pos - model_pos).normalize();
    let new_x = up_vector.cross(&new_z).normalize();
    let new_y = new_z.cross(&new_x).normalize();

    let mut model_mat = Matrix4::identity();
    let model_vec = -1. * model_pos - Point3::origin();
    model_mat.set_column(3, &model_vec.insert_row(3, 1.));

    let view_mat = Matrix4::from_rows(&[
        new_x.transpose().insert_column(3, 0.),
        new_y.transpose().insert_column(3, 0.),
        new_z.transpose().insert_column(3, 0.),
        RowVector4::new(0., 0., 0., 1.),
    ]);

    view_mat * model_mat
}

fn get_projection_matrix(eye_pos: Point3<f32>, model_pos: Point3<f32>) -> Matrix4<f32> {
    Matrix4::<f32>::from_rows(&[
        RowVector4::new(1., 0., 0., 0.),
        RowVector4::new(0., 1., 0., 0.),
        RowVector4::new(0., 0., 1., 0.),
        RowVector4::new(0., 0., -1. / (eye_pos - model_pos).norm(), 1.),
    ])
}

fn get_viewport_matrix(screen_width: u32, screen_height: u32, depth: u32) -> Matrix4<f32> {
    let half_w = ((screen_width - 1) / 2) as f32;
    let half_h = ((screen_height - 1) / 2) as f32;
    let half_d = ((depth - 1) / 2) as f32;
    Matrix4::<f32>::from_rows(&[
        RowVector4::new(half_w, 0., 0., half_w),
        RowVector4::new(0., half_h, 0., half_h),
        RowVector4::new(0., 0., half_d, half_d),
        RowVector4::new(0., 0., 0., 1.),
    ])
}

/// Draw triangle faces of given 3D object
pub fn draw_faces(model: Obj<TexturedVertex>, img: &mut RgbaImage, texture: RgbaImage) {
    let faces_num = model.indices.len();
    let faces = &model.indices[..faces_num];

    let mut z_buffer = vec![vec![f32::NEG_INFINITY; img.height() as usize]; img.width() as usize];

    let camera = Point3::new(1., 1., 3.);
    let model_pos = Point3::new(0., 0., 0.);

    let model_view = get_model_view_matrix(camera, model_pos, Vector3::new(0., 1., 0.));
    let viewport = get_viewport_matrix(img.height(), img.width(), 255);
    let projection = get_projection_matrix(camera, model_pos);

    for face in faces.chunks(3) {
        let mut world_coords = get_face_world_coords(&model, face);
        for coord in world_coords.iter_mut() {
            // println!("Pre {}", coord);
            *coord = Point4::from(projection * model_view * coord.coords);
            *coord /= coord.w;
            // println!("Post {}", coord);
            coord.x = clamp(coord.x, -1.0, 1.0);
            coord.y = clamp(coord.y, -1.0, 1.0);
            coord.z = clamp(coord.z, -1.0, 1.0);
            *coord = Point4::from(viewport * coord.coords);
        }
        let texture_coords = get_face_texture_coords(
            &model,
            face,
            texture.width() as f32,
            texture.height() as f32,
        );
        let normal_coords = get_face_normal_coords(&model, face);

        let light = Vector3::new(0., 0., 1.);
        // Draw face
        draw_face_barycentric(
            &world_coords,
            &texture_coords,
            &texture,
            &normal_coords,
            light,
            &mut z_buffer,
            img,
        );
    }
}
