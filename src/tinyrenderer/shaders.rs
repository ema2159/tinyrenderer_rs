use image::{Pixel, Rgba, RgbaImage};
use nalgebra::{clamp, Matrix2x3, Matrix4, Point2, Point4, Vector2, Vector3};
use obj::{Obj, TexturedVertex};

pub trait Shader {
    fn vertex_shader(&mut self, face: u16, nthvert: usize, gl_position: &mut Point4<f32>);
    fn fragment_shader(&self, bar_coords: Vector3<f32>) -> Option<Rgba<u8>>;
}

fn sample_2d(texture: &RgbaImage, uv: Point2<f32>) -> Rgba<u8> {
    *texture.get_pixel(
        ((uv.x * texture.width() as f32) - 1.) as u32,
        ((uv.y * texture.height() as f32) - 1.) as u32,
    )
}

pub struct RenderingShader<'a> {
    pub model: &'a Obj<TexturedVertex>,
    pub uniform_model_view: Matrix4<f32>,
    pub uniform_model_view_it: Matrix4<f32>,
    pub uniform_projection: Matrix4<f32>,
    pub uniform_viewport: Matrix4<f32>,
    pub uniform_ambient_light: f32,
    pub uniform_texture: RgbaImage,

    pub varying_uv: Matrix2x3<f32>,
}

impl Shader for RenderingShader<'_> {
    fn vertex_shader(&mut self, face_idx: u16, nthvert: usize, gl_position: &mut Point4<f32>) {
        let [u, v, _] = self.model.vertices[face_idx as usize].texture;
        self.varying_uv.set_column(nthvert, &Vector2::new(u, v));

        // Process vertices
        let mv_coords = self.uniform_model_view * gl_position.coords;
        *gl_position = Point4::from(self.uniform_projection * mv_coords);
        *gl_position /= gl_position.w;
        // Clip out of frame points
        gl_position.x = clamp(gl_position.x, -1.0, 1.0);
        gl_position.y = clamp(gl_position.y, -1.0, 1.0);
        *gl_position = Point4::from(self.uniform_viewport * gl_position.coords);
    }
    fn fragment_shader(&self, bar_coords: Vector3<f32>) -> Option<Rgba<u8>> {
        // Texture coords
        let uv = Point2::<f32>::from(self.varying_uv * bar_coords);

        // Fragment calculation
        let mut gl_frag_color = sample_2d(&self.uniform_texture, uv);
        gl_frag_color.apply_without_alpha(|ch| (self.uniform_ambient_light + (ch as f32)) as u8);
        Some(gl_frag_color)
    }
}

pub struct ZShader<'a> {
    pub model: &'a Obj<TexturedVertex>,
    pub uniform_model_view: Matrix4<f32>,
    pub uniform_viewport: Matrix4<f32>,
    pub uniform_projection: Matrix4<f32>,
}

impl Shader for ZShader<'_> {
    fn vertex_shader(&mut self, _face_idx: u16, _nthvert: usize, gl_position: &mut Point4<f32>) {
        // Process vertices
        let mv_coords = self.uniform_model_view * gl_position.coords;
        *gl_position = Point4::from(self.uniform_projection * mv_coords);
        *gl_position /= gl_position.w;
        // Clip out of frame points
        gl_position.x = clamp(gl_position.x, -1.0, 1.0);
        gl_position.y = clamp(gl_position.y, -1.0, 1.0);
        *gl_position = Point4::from(self.uniform_viewport * gl_position.coords);
    }
    fn fragment_shader(&self, _bar_coords: Vector3<f32>) -> Option<Rgba<u8>> {
        None
    }
}
