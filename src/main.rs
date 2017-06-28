extern crate piston_window;

use piston_window::*;

fn main() {
    let mut window: PistonWindow = WindowSettings::new("Hello Piston!", [600; 2]).build().unwrap();
    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g| {
            clear([0.5, 0.5, 0.5, 1.0], g);
            rectangle([0.0, 1.0, 0.0, 1.0],
                      [0.0, 0.0, 50.0, 50.0],
                      c.transform,
                      g);
       
            rectangle([1.0, 0.0, 0.0, 0.7],
                      [0.0, 30.0, 100.0, 100.0],
                      c.transform,
                      g);
        });
    }

}
