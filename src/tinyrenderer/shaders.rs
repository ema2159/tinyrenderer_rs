use image::{Pixel, Rgba, RgbaImage};
use nalgebra::{clamp, Matrix2x3, Matrix3, Matrix4, Point2, Point4, Vector2, Vector3, Vector4};
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
    pub uniform_dir_light: Vector3<f32>,
    pub uniform_texture: RgbaImage,
    pub uniform_normal_map: RgbaImage,
    pub uniform_specular_map: RgbaImage,

    pub varying_uv: Matrix2x3<f32>,
    pub varying_normals: Matrix3<f32>,
    pub varying_ndc_tri: Matrix3<f32>,
}

impl Shader for RenderingShader<'_> {
    fn vertex_shader(&mut self, face_idx: u16, nthvert: usize, gl_position: &mut Point4<f32>) {
        let [u, v, _] = self.model.vertices[face_idx as usize].texture;
        self.varying_uv.set_column(nthvert, &Vector2::new(u, v));

        let [i, j, k] = self.model.vertices[face_idx as usize].normal;
        let normal = (self.uniform_model_view_it * Vector4::new(i, j, k, 0.))
            .xyz()
            .normalize();
        self.varying_normals.set_column(nthvert, &normal);

        *gl_position =
            Point4::from(self.uniform_projection * self.uniform_model_view * gl_position.coords);
        *gl_position /= gl_position.w;
        self.varying_ndc_tri
            .set_column(nthvert, &gl_position.xyz().coords);
        // Clip out of frame points
        gl_position.x = clamp(gl_position.x, -1.0, 1.0);
        gl_position.y = clamp(gl_position.y, -1.0, 1.0);
        *gl_position = Point4::from(self.uniform_viewport * gl_position.coords);
    }
    fn fragment_shader(&self, bar_coords: Vector3<f32>) -> Option<Rgba<u8>> {
        // Texture coords
        let uv = Point2::<f32>::from(self.varying_uv * bar_coords);
        // Normal computing using Darboux tangent space normal mapping
        let bnormal = self.varying_normals * bar_coords;

        let a_row0 = (self.varying_ndc_tri.column(1) - self.varying_ndc_tri.column(0)).transpose();
        let a_row1 = (self.varying_ndc_tri.column(2) - self.varying_ndc_tri.column(0)).transpose();
        let a_row2 = bnormal.transpose();
        let a_inv_mat = Matrix3::from_rows(&[a_row0, a_row1, a_row2])
            .try_inverse()
            .unwrap();

        let i = a_inv_mat
            * Vector3::new(
                self.varying_uv.column(1)[0] - self.varying_uv.column(0)[0],
                self.varying_uv.column(2)[0] - self.varying_uv.column(0)[0],
                0.,
            );

        let j = a_inv_mat
            * Vector3::new(
                self.varying_uv.column(1)[1] - self.varying_uv.column(0)[1],
                self.varying_uv.column(2)[1] - self.varying_uv.column(0)[1],
                0.,
            );

        let b_mat = Matrix3::from_columns(&[i.normalize(), j.normalize(), bnormal]);

        let Rgba([x, y, z, _]) = sample_2d(&self.uniform_normal_map, uv);
        let darboux_mapping = Vector3::new(x, y, z).map(|w| ((w as f32 / 255.) * 2.) - 1.);
        let normal = b_mat * darboux_mapping;

        // Lighting computing
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
            (self.uniform_ambient_light + (ch as f32) * (diffuse + 0.6 * specular)) as u8
        });
        Some(gl_frag_color)
    }
}

pub struct ShadowShader<'a> {
    pub model: &'a Obj<TexturedVertex>,
}

impl Shader for ShadowShader<'_> {
    fn vertex_shader(&mut self, face_idx: u16, nthvert: usize, gl_position: &mut Point4<f32>) {}
    fn fragment_shader(&self, bar_coords: Vector3<f32>) -> Option<Rgba<u8>> {
        Some(Rgba([0, 0, 0, 0]))
    }
}
