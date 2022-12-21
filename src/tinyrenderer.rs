#![allow(dead_code)]
use self::line::draw_line;
use image::{Pixel, Rgba, RgbaImage};
use nalgebra::{clamp, Matrix4, Point2, Point3, Point4, RowVector4, Vector2, Vector3};
use obj::{Obj, TexturedVertex};

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
    let max_x = f32::max(v0_w.x, f32::max(v1_w.x, v2_w.x)) as i32;
    let max_y = f32::max(v0_w.y, f32::max(v1_w.y, v2_w.y)) as i32;
    let min_x = f32::min(v0_w.x, f32::min(v1_w.x, v2_w.x)) as i32;
    let min_y = f32::min(v0_w.y, f32::min(v1_w.y, v2_w.y)) as i32;

    let vec1: Vector2<f32> = (v1_w - v0_w).xy();
    let vec2: Vector2<f32> = (v2_w - v0_w).xy();

    let vec1_x_vec2 = vec1.perp(&vec2) as f32;

    // Calculate if point2 of the bounding box is inside triangle
    for x in min_x..=max_x {
        for y in min_y..=max_y {
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
                if z_buffer[x as usize][y as usize] <= z_value {
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

pub fn get_model_view_matrix(
    eye_pos: Point3<f32>,
    view_point: Point3<f32>,
    model_pos: Point3<f32>,
    model_scale: Vector3<f32>,
    up_vector: Vector3<f32>,
) -> Matrix4<f32> {
    let new_z = (eye_pos - view_point).normalize();
    let new_x = up_vector.cross(&new_z).normalize();
    let new_y = new_z.cross(&new_x).normalize();

    let mut model_mat = Matrix4::from_diagonal(&model_scale.insert_row(3, 1.));
    let eye_vec = model_pos - eye_pos;
    model_mat.set_column(3, &(eye_vec.insert_row(3, 1.)));

    let view_mat = Matrix4::from_rows(&[
        new_x.transpose().insert_column(3, 0.),
        new_y.transpose().insert_column(3, 0.),
        new_z.transpose().insert_column(3, 0.),
        RowVector4::new(0., 0., 0., 1.),
    ]);

    view_mat * model_mat
}

pub fn get_projection_matrix(f: f32) -> Matrix4<f32> {
    Matrix4::<f32>::from_rows(&[
        RowVector4::new(1., 0., 0., 0.),
        RowVector4::new(0., 1., 0., 0.),
        RowVector4::new(0., 0., 1., 0.),
        RowVector4::new(0., 0., -1. / f, 1.),
    ])
}

pub fn get_viewport_matrix(screen_width: f32, screen_height: f32, depth: f32) -> Matrix4<f32> {
    let half_w = (screen_width - 1.) / 2.;
    let half_h = (screen_height - 1.) / 2.;
    let half_d = (depth - 1.) / 2.;
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

    // Screen properties
    let (width, height) = (img.width() as f32, img.height() as f32);

    // Model configuration
    let model_pos = Point3::new(0., 0., 0.);
    let model_scale = Vector3::new(1., 1., 1.);

    // Camera and light configuration
    let camera = Point3::new(0.5, 0.5, 1.);
    let view_point = model_pos;
    let light = Vector3::new(0., 0., 1.);

    // Transformation matrices
    let model_view = get_model_view_matrix(
        camera,
        view_point,
        model_pos,
        model_scale,
        Vector3::new(0., 1., 0.),
    );
    let viewport = get_viewport_matrix(height, width, 255.);
    let projection = get_projection_matrix(3.);

    let mut z_buffer = vec![vec![f32::NEG_INFINITY; img.height() as usize]; img.width() as usize];

    for face in faces.chunks(3) {
        let mut world_coords = get_face_world_coords(&model, face);
        for coord in world_coords.iter_mut() {
            *coord = Point4::from(projection * model_view * coord.coords);
            *coord /= coord.w;
            // Clip out of frame points
            coord.x = clamp(coord.x, -1.0, 1.0);
            coord.y = clamp(coord.y, -1.0, 1.0);
            *coord = Point4::from(viewport * coord.coords);
        }
        let texture_coords = get_face_texture_coords(
            &model,
            face,
            texture.width() as f32,
            texture.height() as f32,
        );
        let normal_coords = get_face_normal_coords(&model, face);

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

mod line;
