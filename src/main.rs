extern crate piston_window;
extern crate specs;

use piston_window::*;
use specs::{Component, DispatcherBuilder, Dispatcher, ReadStorage, System, VecStorage, World, WriteStorage};


#[derive(Debug)]
struct Pos {
    x: f64,
    y: f64,
}

struct Delta(f64);

struct Game<'a> {
    world: World,
    //TODO: are these lifetimes right?
    dispatcher: Dispatcher<'a, 'a>,
}

impl<'a> Game<'a> {
    fn new() -> Game<'a> {
        let mut world = World::new();
        world.add_resource(Delta(0.0));

        let dispatcher = DispatcherBuilder::new().build();
        Game {
            world: world,
            dispatcher: dispatcher,
        }
    }
    fn update(&mut self, d: f64) {
        {
            let mut delta = self.world.write_resource::<Delta>();
            *delta = Delta(d);
        }
        self.dispatcher.dispatch(&mut self.world.res);
        self.world.maintain();
    }
}

fn main() {
    let mut pos = Pos { x: 0.0, y: 0.0 };
    let mut window: PistonWindow = WindowSettings::new("Hello Piston!", [600; 2]).build().unwrap();

    let mut game = Game::new();

    while let Some(e) = window.next() {
        match e {
            Input::Update(UpdateArgs { dt: delta }) => {
                game.update(delta);
                update_state(&e, &mut pos);

            }
            Input::Render(_) => {
                window.draw_2d(&e, |c, mut g| {
                    render_game(&pos, c, &mut g);
                });
            }
            _ => {}
        }
    }
}
fn render_game(pos: &Pos, c: Context, g: &mut G2d) {
    clear([0.5, 0.5, 0.5, 1.0], g);
    rectangle([0.0, 1.0, 0.0, 1.0], [0.0, 0.0, 50.0, 50.0], c.transform, g);
    rectangle([1.0, 0.0, 0.0, 0.7],
              [pos.x, pos.y, 100.0, 100.0],
              c.transform,
              g);
}
fn update_state<E>(e: &E, pos: &mut Pos)
    where E: GenericEvent
{
    e.press(|b| {
        pos.x += 1.0;
    });
}