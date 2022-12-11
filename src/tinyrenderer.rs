#![allow(dead_code)]
pub mod gl;
pub mod line;
pub mod shaders;

use self::shaders::Shader;
use image::RgbaImage;
use nalgebra::{Point2, Point4, Vector2, Vector3};
use obj::{Obj, TexturedVertex};

/// Implementation of barycentric algorithm for triangle filling. Works as the rasterizer.
fn draw_face_barycentric(
    screen_coords: [Point4<f32>; 3],
    shaders: &dyn Shader,
    color_buffer: &mut RgbaImage,
    z_buffer: &mut [Vec<f32>],
) {
    let [v0_w, v1_w, v2_w] = screen_coords;
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
            let pv0 = Point2::<f32>::new(x as f32, y as f32) - v0_w.xy();
            let vec1_x_pv0 = vec1.perp(&pv0) as f32;
            let pv0_x_vec2 = pv0.perp(&vec2) as f32;
            // Barycentric coordinates
            let s = vec1_x_pv0 / vec1_x_vec2;
            let t = pv0_x_vec2 / vec1_x_vec2;
            let t_s_1 = 1. - (t + s);
            let bar_coords = Vector3::<f32>::new(t_s_1, t, s);

            if s >= 0. && t >= 0. && t_s_1 >= 0. {
                let z_value = t_s_1 * v0_w.z + t * v1_w.z + s * v2_w.z;
                if z_buffer[x as usize][y as usize] < z_value {
                    z_buffer[x as usize][y as usize] = z_value;
                    if let Some(frag) = shaders.fragment_shader(bar_coords) {
                        color_buffer.put_pixel(x as u32, y as u32, frag);
                    }
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

/// Draw triangle faces of given 3D object. Works as the primitive processor.
pub fn draw_faces(
    model: &Obj<TexturedVertex>,
    color_buffer: &mut RgbaImage,
    z_buffer: &mut [Vec<f32>],
    shaders: &mut dyn Shader,
) {
    let faces_num = model.indices.len();
    let faces = &model.indices[..faces_num];

    for face in faces.chunks(3) {
        let mut verts = get_face_world_coords(model, face);
        for (i, vert) in verts.iter_mut().enumerate() {
            shaders.vertex_shader(face[i], i, vert);
        }

        // Draw face
        draw_face_barycentric(verts, shaders, color_buffer, z_buffer);
    }
}
