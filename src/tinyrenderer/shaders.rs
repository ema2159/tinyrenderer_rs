use image::{Pixel, Rgba, RgbaImage};
use nalgebra::{clamp, Matrix2x3, Matrix4, Point2, Point4, Vector2, Vector3, Vector4};
use obj::{Obj, TexturedVertex};

pub trait Shader {
    fn vertex_shader(&mut self, face: u16, nthvert: usize, gl_position: &mut Point4<f32>);
    fn fragment_shader(&self, bar_coords: Vector3<f32>) -> Option<Rgba<u8>>;
}

pub struct MyShader<'a> {
    pub model: &'a Obj<TexturedVertex>,
    pub uniform_model_view: Matrix4<f32>,
    pub uniform_model_view_it: Matrix4<f32>,
    pub uniform_projection: Matrix4<f32>,
    pub uniform_viewport: Matrix4<f32>,
    pub uniform_ambient_light: f32,
    pub uniform_dir_light: Vector3<f32>,
    pub uniform_texture: RgbaImage,
    pub uniform_normal_map: RgbaImage,
    pub uniform_specular_map: RgbaImage,

    pub varying_uv: Matrix2x3<f32>,
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

        *gl_position =
            Point4::from(self.uniform_projection * self.uniform_model_view * gl_position.coords);
        *gl_position /= gl_position.w;
        // Clip out of frame points
        gl_position.x = clamp(gl_position.x, -1.0, 1.0);
        gl_position.y = clamp(gl_position.y, -1.0, 1.0);
        *gl_position = Point4::from(self.uniform_viewport * gl_position.coords);
    }
    fn fragment_shader(&self, bar_coords: Vector3<f32>) -> Option<Rgba<u8>> {
        // Texture coords
        let uv = Point2::<f32>::from(self.varying_uv * bar_coords);
        // Normal computing
        let Rgba([i, j, k, _]) = sample_2d(&self.uniform_normal_map, uv);
        let normal_map = Vector4::new(i, j, k, 255).map(|x| ((x as f32 / 255.) * 2.) - 1.);
        let normal = (self.uniform_model_view_it * normal_map).xyz().normalize();
        let reflected = (normal * (normal.dot(&self.uniform_dir_light) * 2.)
            - self.uniform_dir_light)
            .normalize();

        let specular = f32::powi(
            f32::max(0., reflected.z),
            sample_2d(&self.uniform_specular_map, uv)[0].into(),
        );
        let diffuse = f32::max(0., self.uniform_dir_light.dot(&normal));

        // Fragment calculation
        let mut gl_frag_color = sample_2d(&self.uniform_texture, uv);
        gl_frag_color.apply_without_alpha(|ch| {
            (self.uniform_ambient_light + (ch as f32) * (diffuse + specular)) as u8
        });
        Some(gl_frag_color)
    }
}
