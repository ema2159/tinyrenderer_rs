use nalgebra::{clamp, Matrix4, Point3, Point4, Vector3};

pub trait Shader {
    fn vertex_shader(&self, vert: &Point4<f32>) -> Point3<f32>;
    fn fragment_shader(&self);
}

pub struct MyShader {
    pub uniform_model_view_mat: Matrix4<f32>,
    pub uniform_projection_mat: Matrix4<f32>,
    pub uniform_viewport_mat: Matrix4<f32>,
    pub uniform_light: Vector3<f32>,

    pub varying_intensity: f32,
}

impl Shader for MyShader {
    fn vertex_shader(&self, coord: &Point4<f32>) -> Point3<f32> {
        let mut screen_coord =
            Point4::from(self.uniform_projection_mat * self.uniform_model_view_mat * coord.coords);
        screen_coord /= screen_coord.w;
        // Clip out of frame points
        screen_coord.x = clamp(screen_coord.x, -1.0, 1.0);
        screen_coord.y = clamp(screen_coord.y, -1.0, 1.0);
        screen_coord = Point4::from(self.uniform_viewport_mat * screen_coord.coords);
        screen_coord.xyz()
    }
    fn fragment_shader(&self) {}
}
