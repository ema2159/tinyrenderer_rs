extern crate image;
extern crate nalgebra;
extern crate obj;
extern crate piston_window;

mod tinyrenderer;

use image::{Rgba, RgbaImage};
use obj::{load_obj, Obj, TexturedVertex};
use piston_window::EventLoop;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tinyrenderer::draw_faces;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

fn main() -> Result<(), Box<dyn Error>> {
    let mut img = RgbaImage::from_pixel(WIDTH, HEIGHT, Rgba([0, 0, 0, 255]));

    // Assets dir
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        panic!("No assets directory provided!");
    }
    let assets_dir = Path::new(&args[1])
        .canonicalize()
        .unwrap_or_else(|_| panic!("Wrong path for assets directory!"));

    // Load model
    let obj_path = assets_dir.join("african_head.obj");
    let input = BufReader::new(File::open(&obj_path)?);
    let model: Obj<TexturedVertex> = load_obj(input)?;

    // Load texture
    let texture_path = assets_dir.join("african_head_diffuse.tga");
    let mut texture = image::open(texture_path)
        .expect("Opening image failed")
        .into_rgba8();

    image::imageops::flip_vertical_in_place(&mut texture);

    use std::time::Instant;
    let now = Instant::now();
    draw_faces(model, &mut img, texture);
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    image::imageops::flip_vertical_in_place(&mut img);

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
        &img,
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
