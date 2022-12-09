use image::{Rgba, RgbaImage};
use nalgebra::{clamp, Matrix2x3, Matrix4, Point2, Point4, Vector2, Vector3};
use obj::{Obj, TexturedVertex};

pub trait Shader {
    fn vertex_shader(&mut self, face: u16, nthvert: usize, gl_position: &mut Point4<f32>);
    fn fragment_shader(&self, bar_coords: Vector3<f32>) -> Option<Rgba<u8>>;
}

pub struct MyShader<'a> {
    pub model: &'a Obj<TexturedVertex>,
    pub uniform_model_view_mat: Matrix4<f32>,
    pub uniform_projection_mat: Matrix4<f32>,
    pub uniform_viewport_mat: Matrix4<f32>,
    pub uniform_light: Vector3<f32>,
    pub uniform_texture: RgbaImage,

    pub varying_uv: Matrix2x3<f32>,
    pub varying_intensity: [Point2<f32>; 3],
}

fn sample_2d(texture: &RgbaImage, uv: Point2<f32>) -> Rgba<u8> {
    *texture.get_pixel(
        ((uv.x * texture.width() as f32) - 1.) as u32,
        ((uv.y * texture.height() as f32) - 1.) as u32,
    )
}

impl Shader for MyShader<'_> {
    fn vertex_shader(&mut self, face_idx: u16, nthvert: usize, gl_position: &mut Point4<f32>) {
        let [u, v, _] = self.model.vertices[face_idx as usize].texture;
        self.varying_uv.set_column(nthvert, &Vector2::new(u, v));

        *gl_position = Point4::from(
            self.uniform_projection_mat * self.uniform_model_view_mat * gl_position.coords,
        );
        *gl_position /= gl_position.w;
        // Clip out of frame points
        gl_position.x = clamp(gl_position.x, -1.0, 1.0);
        gl_position.y = clamp(gl_position.y, -1.0, 1.0);
        *gl_position = Point4::from(self.uniform_viewport_mat * gl_position.coords);
    }
    fn fragment_shader(&self, bar_coords: Vector3<f32>) -> Option<Rgba<u8>> {
        let uv = Point2::<f32>::from(self.varying_uv * bar_coords);
        let gl_frag_color = sample_2d(&self.uniform_texture, uv);
        // tex_point.apply_without_alpha(|ch| ((ch as f32) * light_intensity) as u8);
        Some(gl_frag_color)
    }
}
