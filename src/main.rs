extern crate piston_window;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

fn main() {
    let mut window: piston_window::PistonWindow =
        piston_window::WindowSettings::new("tinyrenderer_rs", [WIDTH, HEIGHT])
            .exit_on_esc(true)
            .build()
            .unwrap_or_else(|_e| panic!("Could not create window!"));
    while let Some(_event) = window.next() {
    }
}
