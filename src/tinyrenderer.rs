#![allow(dead_code)]
pub mod gl;
mod line;
pub mod shaders;

use self::{
    line::draw_line,
    shaders::{MyShader, Shader},
};
use image::{Pixel, Rgba, RgbaImage};
use nalgebra::{Point2, Point3, Point4, Vector2, Vector3};
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

/// Implementation of barycentric algorithm for triangle filling. Works as the rasterizer.
fn draw_face_barycentric(
    screen_coords: &[Point3<f32>; 3],
    texture_coords: &[Point2<f32>; 3],
    texture: &RgbaImage,
    normal_coords: &[Vector3<f32>; 3],
    shaders: &mut MyShader,
    img: &mut RgbaImage,
) {
    let [v0_w, v1_w, v2_w] = &screen_coords;
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
                let light_intensity = normal.dot(&shaders.light);
                if shaders.z_buffer[x as usize][y as usize] < z_value {
                    shaders.z_buffer[x as usize][y as usize] = z_value;
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

/// Draw triangle faces of given 3D object. Works as the primitive processor.
pub fn draw_faces(
    model: Obj<TexturedVertex>,
    img: &mut RgbaImage,
    texture: RgbaImage,
    shaders: &mut MyShader,
) {
    let faces_num = model.indices.len();
    let faces = &model.indices[..faces_num];

    for face in faces.chunks(3) {
        let mut world_coords = get_face_world_coords(&model, face);
        let mut screen_coords = [Point3::<f32>::origin(); 3];
        for (i, coord) in world_coords.iter_mut().enumerate() {
            screen_coords[i] = shaders.vertex_shader(coord);
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
            &screen_coords,
            &texture_coords,
            &texture,
            &normal_coords,
            shaders,
            img,
        );
    }
}
