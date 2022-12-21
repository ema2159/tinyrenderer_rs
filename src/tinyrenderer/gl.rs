use nalgebra::{Matrix4, Point2, Point3, RowVector4, Vector2, Vector3};

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

pub fn max_elevation_angle(
    z_buffer: &Vec<Vec<f32>>,
    center_point: Point2<f32>,
    dir: Vector2<f32>,
) -> f32 {
    let (width, height) = (z_buffer.len(), z_buffer[0].len());
    let (x0, y0) = (center_point.x as usize, center_point.y as usize);

    let mut max_angle: f32 = 0.;
    const RAY_LENGTH: usize = 1000;
    const RAY_STEP: usize = 5;
    for t in (0..RAY_LENGTH).step_by(RAY_STEP) {
        let curr_point = center_point + (t as f32) * dir;
        let (x, y) = (curr_point.x as usize, curr_point.y as usize);
        if !((0..width).contains(&x) && (0..height).contains(&y)) {
            return max_angle;
        }

        let distance = (center_point - curr_point).norm();
        if distance < 1. {
            continue;
        };
        let elevation = z_buffer[x][y] - z_buffer[x0][y0];
        max_angle = f32::max(max_angle, f32::atan(elevation / distance));
    }
    max_angle
}
