use nalgebra::{clamp, Matrix4, Point3, Point4, Vector3};

pub trait Shader {
    fn vertex_shader(&self, vert: &Point4<f32>) -> Point3<f32>;
    fn fragment_shader(&self);
}

pub struct MyShader {
    pub model_view_matrix: Matrix4<f32>,
    pub projection_matrix: Matrix4<f32>,
    pub viewport_matrix: Matrix4<f32>,
    pub light: Vector3<f32>,

    pub varying_intensity: f32,
}

impl Shader for MyShader {
    fn vertex_shader(&self, coord: &Point4<f32>) -> Point3<f32> {
        let mut screen_coord =
            Point4::from(self.projection_matrix * self.model_view_matrix * coord.coords);
        screen_coord /= screen_coord.w;
        // Clip out of frame points
        screen_coord.x = clamp(screen_coord.x, -1.0, 1.0);
        screen_coord.y = clamp(screen_coord.y, -1.0, 1.0);
        screen_coord = Point4::from(self.viewport_matrix * screen_coord.coords);
        screen_coord.xyz()
    }
    fn fragment_shader(&self) {}
}
