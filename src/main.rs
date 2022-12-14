extern crate image;
extern crate nalgebra;
extern crate obj;
extern crate piston_window;

mod tinyrenderer;

use image::{Rgba, RgbaImage};
use nalgebra::{Matrix2x3, Matrix3, Point3, Vector3};
use obj::{load_obj, Obj, TexturedVertex};
use piston_window::EventLoop;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tinyrenderer::draw_faces;
use tinyrenderer::gl::{get_model_view_matrix, get_projection_matrix, get_viewport_matrix};
use tinyrenderer::shaders::{RenderingShader, ShadowShader};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

pub struct Camera {
    pub position: Point3<f32>,
    pub focal_length: f32,
    pub view_point: Point3<f32>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut color_buffer = RgbaImage::from_pixel(WIDTH, HEIGHT, Rgba([0, 0, 0, 255]));
    let mut _buffer = RgbaImage::from_pixel(WIDTH, HEIGHT, Rgba([0, 0, 0, 255]));

    // Assets dir
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        panic!("No assets directory provided!");
    }
    let assets_dir = Path::new(&args[1])
        .canonicalize()
        .unwrap_or_else(|_| panic!("Wrong path for assets directory!"));

    // Load model
    let obj_path = assets_dir.join("diablo3_pose.obj");
    let input = BufReader::new(File::open(obj_path)?);
    let model: Obj<TexturedVertex> = load_obj(input)?;

    // Load texture
    let texture_path = assets_dir.join("diablo3_pose_diffuse.tga");
    let mut texture = image::open(texture_path)
        .expect("Opening image failed")
        .into_rgba8();
    image::imageops::flip_vertical_in_place(&mut texture);

    // Load normal map
    let normal_map_path = assets_dir.join("diablo3_pose_nm_tangent.tga");
    let mut normal_map = image::open(normal_map_path)
        .expect("Opening image failed")
        .into_rgba8();
    image::imageops::flip_vertical_in_place(&mut normal_map);

    // Load specular map
    let specular_map_path = assets_dir.join("diablo3_pose_spec.tga");
    let mut specular_map = image::open(specular_map_path)
        .expect("Opening image failed")
        .into_rgba8();
    image::imageops::flip_vertical_in_place(&mut specular_map);

    use std::time::Instant;
    let now = Instant::now();

    // Frame properties
    let (width, height) = (color_buffer.width() as f32, color_buffer.height() as f32);
    let depth = 1024.;

    // Model configuration
    let model_pos = Point3::new(0., 0., 0.);
    let model_scale = Vector3::new(1., 1., 1.);

    // Camera configuration
    let camera = Camera {
        position: Point3::new(0., 0., 1.),
        focal_length: 3.,
        view_point: model_pos,
    };

    // Light configuration
    let ambient_light = 5.;
    let dir_light = Vector3::new(-1., 0., 1.5);

    // Z buffer
    let mut z_buffer = vec![vec![f32::NEG_INFINITY; height as usize]; width as usize];

    // Shadow buffer
    let mut shadow_buffer = vec![vec![f32::NEG_INFINITY; height as usize]; width as usize];

    // Transformation matrices
    let model_view = get_model_view_matrix(
        camera.position,
        camera.view_point,
        model_pos,
        model_scale,
        Vector3::new(0., 1., 0.),
    );
    let projection = get_projection_matrix(camera.focal_length);
    let model_view_it = model_view.try_inverse().unwrap().transpose();
    let viewport = get_viewport_matrix(height, width, depth);

    let shadow_mat = get_model_view_matrix(
        Point3::<f32>::origin() + dir_light,
        model_pos,
        model_pos,
        model_scale,
        Vector3::new(0., 1., 0.),
    );

    // Shaders
    let mut shadow_shader = ShadowShader {
        model: &model,
        uniform_shadow_mv_mat: shadow_mat,
        uniform_viewport: viewport,

        varying_view_tri: Matrix3::<f32>::zeros(),
    };
    // Compute shadows
    draw_faces(&model, &mut _buffer, &mut shadow_buffer, &mut shadow_shader);

    let mut rendering_shader = RenderingShader {
        model: &model,
        shadow_buffer: &shadow_buffer,
        uniform_model_view: model_view,
        uniform_model_view_it: model_view_it,
        uniform_shadow_mv_mat: shadow_mat,
        uniform_projection: projection,
        uniform_viewport: viewport,
        uniform_ambient_light: ambient_light,
        uniform_dir_light: (model_view * dir_light.insert_row(3, 0.)).xyz().normalize(),
        uniform_texture: texture,
        uniform_normal_map: normal_map,
        uniform_specular_map: specular_map,

        varying_uv: Matrix2x3::<f32>::zeros(),
        varying_normals: Matrix3::<f32>::zeros(),
        varying_view_tri: Matrix3::<f32>::zeros(),
        varying_shadow_tri: Matrix3::<f32>::zeros(),
    };

    // Render model
    draw_faces(
        &model,
        &mut color_buffer,
        &mut z_buffer,
        &mut rendering_shader,
    );

    image::imageops::flip_vertical_in_place(&mut color_buffer);

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    // Rendering window
    let mut window: piston_window::PistonWindow =
        piston_window::WindowSettings::new("tinyrenderer_rs", [WIDTH, HEIGHT])
            .exit_on_esc(true)
            .build()
            .unwrap_or_else(|_e| panic!("Could not create window!"));

    // Configure window properties
    window.set_lazy(true);
    window.set_max_fps(60);

    let rendered_img = piston_window::Texture::from_image(
        &mut window.create_texture_context(),
        &color_buffer,
        &piston_window::TextureSettings::new(),
    )
    .unwrap();

    while let Some(event) = window.next() {
        window.draw_2d(&event, |c, g, _| {
            piston_window::clear([0.0, 0.0, 0.0, 1.0], g);
            piston_window::image(&rendered_img, c.transform, g);
        });
    }
    Ok(())
}
