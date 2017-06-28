extern crate piston_window;

use piston_window::*;

#[derive(Debug)]
struct Pos {
    x: f64,
    y: f64,
}

fn main() {
    let mut pos = Pos { x: 0.0, y: 0.0 };
    let mut window: PistonWindow = WindowSettings::new("Hello Piston!", [600; 2]).build().unwrap();
    while let Some(e) = window.next() {
        update_state(&e, &mut pos);
        window.draw_2d(&e, |c, g| {
            clear([0.5, 0.5, 0.5, 1.0], g);
            rectangle([0.0, 1.0, 0.0, 1.0], [0.0, 0.0, 50.0, 50.0], c.transform, g);

            rectangle([1.0, 0.0, 0.0, 0.7],
                      [pos.x, pos.y, 100.0, 100.0],
                      c.transform,
                      g);
        });
    }
}

fn update_state<E>(e : &E, pos : &mut Pos)
    where E : GenericEvent
{
    e.press(|b| {
        pos.x += 1.0;
    });
}