use nalgebra::{clamp, Matrix4, Point4, Vector3};

pub trait Shader {
    fn vertex_shader(&self, vert: &mut Point4<f32>);
    fn fragment_shader(&self);
}

pub struct MyShader<'a> {
    pub model_view_matrix: Matrix4<f32>,
    pub projection_matrix: Matrix4<f32>,
    pub viewport_matrix: Matrix4<f32>,
    pub light: Vector3<f32>,
    pub z_buffer: &'a mut Vec<Vec<f32>>,

    pub varying_intensity: f32,
}

impl Shader for MyShader<'_> {
    fn vertex_shader(&self, coord: &mut Point4<f32>) {
        *coord = Point4::from(self.projection_matrix * self.model_view_matrix * coord.coords);
        *coord /= coord.w;
        // Clip out of frame points
        coord.x = clamp(coord.x, -1.0, 1.0);
        coord.y = clamp(coord.y, -1.0, 1.0);
        *coord = Point4::from(self.viewport_matrix * coord.coords);
    }
    fn fragment_shader(&self) {}
}
