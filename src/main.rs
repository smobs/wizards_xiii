extern crate piston_window;
extern crate specs;
extern crate ncollide;

use piston_window::*;
use piston_window::Button::Keyboard;
use specs::{DispatcherBuilder, Dispatcher, ReadStorage, System, VecStorage, World, WriteStorage,
            FetchMut, Join};
use std::collections::HashSet;
use std::ops::DerefMut;
mod systems;
use systems::assorted::*;
use systems::components::*;
use systems::collision::*;

struct Game<'a> {
    world: World,
    // TODO: are these lifetimes right?
    dispatcher: Dispatcher<'a, 'a>,
}

impl<'a> Game<'a> {
    fn new() -> Game<'a> {
        let mut world = World::new();
        world.add_resource(Delta(0.0));
        world.add_resource(GameInput(HashSet::new()));

        world.register::<Pos>();
        world.register::<Vel>();
        world.register::<Bounds>();
        world.register::<Player>();
        world.register::<CollisionObjectData>();

        world.create_entity()
            .with(Pos { x: 50.0, y: 50.0 })
            .with(Vel { x: 0.0, y: 0.0 })
            .with(Player(1))
            .with(Bounds::Rectangle(50.0,50.0))
            .with(CollisionObjectData {});
        world.create_entity()
            .with(Pos { x: 100.0, y: 50.0 })
            .with(Vel { x: 0.0, y: 0.0 })
            .with(Bounds::Circle(25.0))
            .with(Player(2))
            .with(CollisionObjectData {});
        
        world.create_entity()
            .with(Pos { x: 200.0, y: 200.0 })
            .with(Vel { x: 0.0, y: 0.0 })
            .with(Bounds::Rectangle(100.0, 50.0))
            .with(CollisionObjectData {});

        let dispatcher = DispatcherBuilder::new()
            .add(UpdateControlSystem, "ControlSystem", &[])
            .add(UpdatePositionSystem,
                 "UpdatePositionSystem",
                 &["ControlSystem"])
            .add_thread_local(CollisionSystem::new())
            .build();
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
    fn keypress(&mut self, button: Button) {
        let mut input_set = self.world.write_resource::<GameInput>();
        let x = input_set.deref_mut();
        x.0.insert(button);
    }
    fn keyrelease(&mut self, button: Button) {
        let mut input_set = self.world.write_resource::<GameInput>();
        let x = input_set.deref_mut();
        x.0.remove(&button);
    }
    fn render(&self, c: Context, g: &mut G2d) {
        let pos = &self.world.read::<Pos>();
        let bounds = &self.world.read::<Bounds>();
        clear([0.5, 0.5, 0.5, 1.0], g);
        for (pos, bounds) in (pos, bounds).join() { 
            match *bounds {
                Bounds::Rectangle(x, y) => {
                    rectangle([1.0, 0.0, 0.0, 0.7], [pos.x - (x / 2.0), pos.y -(y / 2.0), x, y], c.transform, g);
                }
                Bounds::Circle(r) => {
                    ellipse([0.0, 0.0, 1.0, 0.7],
                            [pos.x - r, pos.y - r, 2.0*r, 2.0*r],
                            c.transform,
                            g);
                }
            }
        }
    }
}

fn main() {
    let mut window: PistonWindow =
        WindowSettings::new("Hello Piston!", [700, 500]).build().unwrap();

    let mut game = Game::new();

    while let Some(e) = window.next() {
        match e {
            Input::Update(UpdateArgs { dt: delta }) => {
                game.update(delta);
            }
            Input::Render(_) => {
                window.draw_2d(&e, |c, mut g| {
                    game.render(c, &mut g);
                });
            }
            Input::Press(button) => {
                game.keypress(button);
            }
            Input::Release(button) => {
                game.keyrelease(button);
            }
            _ => {}
        }
    }
}