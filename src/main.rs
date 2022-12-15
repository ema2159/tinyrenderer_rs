extern crate image;
extern crate obj;
extern crate piston_window;

mod tinyrenderer;

use image::{Rgba, RgbaImage};
use obj::{load_obj, Obj};
use piston_window::EventLoop;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tinyrenderer::draw_wireframe;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

fn main() -> Result<(), Box<dyn Error>> {
    let mut img = RgbaImage::from_pixel(WIDTH, HEIGHT, Rgba([0, 0, 0, 255]));

    let obj_path = Path::new(
        "/home/ema2159/Documents/GitHub/tinyrenderer_rs/assets/african_head/african_head.obj",
    );
    let input = BufReader::new(File::open(&obj_path)?);
    let model: Obj = load_obj(input)?;

    draw_wireframe(model, Rgba([255, 255, 255, 255]), &mut img);

    image::imageops::flip_vertical_in_place(&mut img);

    let mut window: piston_window::PistonWindow =
        piston_window::WindowSettings::new("tinyrenderer_rs", [WIDTH, HEIGHT])
            .exit_on_esc(true)
            .build()
            .unwrap_or_else(|_e| panic!("Could not create window!"));

    window.set_lazy(true);

    let texture = piston_window::Texture::from_image(
        &mut window.create_texture_context(),
        &img,
        &piston_window::TextureSettings::new(),
    )
    .unwrap();

    while let Some(event) = window.next() {
        window.draw_2d(&event, |c, g, _| {
            piston_window::clear([0.0, 0.0, 0.0, 1.0], g);
            piston_window::image(&texture, c.transform, g);
        });
    }
    Ok(())
}
