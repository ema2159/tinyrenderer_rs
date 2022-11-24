extern crate image;
extern crate piston_window;

mod tinyrenderer;

use image::{Rgba, RgbaImage};
use piston_window::EventLoop;
use tinyrenderer::draw_line;
use tinyrenderer::Point;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

fn main() {
    let mut img = RgbaImage::from_pixel(WIDTH, HEIGHT, Rgba([0, 0, 0, 255]));

    draw_line(
        &mut img,
        Point { x: 0, y: 0 },
        Point { x: 700, y: 719 },
        Rgba([255, 255, 255, 255]),
    );

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
}
