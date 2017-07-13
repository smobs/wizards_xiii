extern crate piston_window;
extern crate specs;
extern crate ncollide;
extern crate nalgebra;

use piston_window::*;
use piston_window::Button::Keyboard;
use specs::{DispatcherBuilder, Dispatcher, ReadStorage, System, VecStorage, World, WriteStorage,
            FetchMut, Join};
use std::collections::HashSet;
use std::ops::DerefMut;
use std::iter::*;

mod systems;
use systems::assorted::*;
use systems::components::*;
use systems::collision::*;
use systems::terrain::*;

struct Game<'a> {
    world: World,
    // TODO: are these lifetimes right?
    dispatcher: Dispatcher<'a, 'a>,
}
fn create_terrain(world: &mut World) {
    world.create_entity()
        .with(Pos { x: 0.0, y: 0.0 })
        .with(Bounds::Polygon(Box::new(vec![])))
        .with(CollisionObjectData::new(3))
        .with(Terrain::new(200, 400, 500, 100));
}
fn create_players(world: &mut World) {
    world.create_entity()
        .with(Pos {
            x: 350.0,
            y: 100.0,
        })
        .with(Vel { x: 0.0, y: 0.0 })
        .with(Player(1))
        .with(Bounds::Rectangle(50.0, 50.0))
        .with(CollisionObjectData::new(1));
    world.create_entity()
        .with(Pos {
            x: 400.0,
            y: 50.0,
        })
        .with(Vel { x: 0.0, y: 0.0 })
        .with(Bounds::Circle(25.0))
        .with(Player(2))
        .with(CollisionObjectData::new(2));

}

fn draw_bounds(bounds : &Bounds, pos : &Pos, c: Context, g: &mut G2d ) {
    match bounds {
        &Bounds::Rectangle(x, y) => {
            rectangle([1.0, 0.0, 0.0, 1.0],
                      [pos.x - (x / 2.0), pos.y - (y / 2.0), x, y],
                      c.transform,
                      g);
        }
        &Bounds::Circle(r) => {
            ellipse([0.0, 0.0, 1.0, 1.0],
                    [pos.x - r, pos.y - r, 2.0 * r, 2.0 * r],
                    c.transform,
                    g);
        }
        &Bounds::Polygon(ref ps) => {
            let ps = Vec::from_iter(ps[..].into_iter().map(|p| [p[0] + pos.x, p[1] + pos.y]));

            polygon([0.0, 1.0, 0.0, 0.5], &ps, c.transform, g)
        }
        &Bounds::Grid{points: ref ps, height: _, width: _} => {
            for p in ps.iter() {
                let temp = Pos{
                    x: pos.x + (p[0] as f64),
                    y: pos.y + (p[1] as f64),
                };
                draw_bounds(&Bounds::Rectangle(1.0, 1.0),  &temp, c, g)
            }
        }
    }
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
        world.register::<Terrain>();

        create_players(&mut world);

        create_terrain(&mut world);

        let dispatcher = DispatcherBuilder::new()
            .add(TerrainSystem, "TerrainSystem", &[])
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
            draw_bounds(bounds, pos, c, g)
        }
        for col in (&self.world.read::<CollisionObjectData>())
            .join()
            .flat_map(|c| c.contacts.values().flat_map(|v| v)) {
            let r = 10.0;
            ellipse([0.0, 1.0, 1.0, 1.0],
                    [col[0] - r, col[1] - r, 2.0 * r, 2.0 * r],
                    c.transform,
                    g);
        }
    }
}

fn main() {
    let mut window: PistonWindow =
        WindowSettings::new("Hello Piston!", [700, 500]).exit_on_esc(true).build().unwrap();

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