extern crate piston_window;
extern crate specs;

use piston_window::*;
use specs::{DispatcherBuilder, Dispatcher, ReadStorage, System, VecStorage, World, WriteStorage, FetchMut, Join};

mod systems;
use systems::*;

struct Delta(f64);

struct RenderSys;

struct Game<'a> {
    world: World,
    // TODO: are these lifetimes right?
    dispatcher: Dispatcher<'a, 'a>,
}

impl<'a> Game<'a> {
    fn new(window : &PistonWindow ) -> Game<'a> {
        let mut world = World::new();
        world.add_resource(Delta(0.0));
        world.register::<Pos>();
        world.register::<Vel>();

        world.create_entity().with(Pos{x:1.0, y:0.0});
        world.create_entity().with(Pos{x:100.0, y:0.0}).with(Vel{x: 0.0, y: 1.0});
        
        let dispatcher = DispatcherBuilder::new().add(UpdatePositionSystem, "UpdatePositionSystem", &[]).build();
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
    fn render(&self, c: Context, g: &mut G2d){
        let pos = &self.world.read::<Pos>();
        clear([0.5, 0.5, 0.5, 1.0], g);
        for pos in pos.join(){
        rectangle([1.0, 0.0, 0.0, 0.7],
              [pos.x, pos.y, 100.0, 100.0],
              c.transform,
              g);
        }
    }
}

fn main() {
    let mut pos = Pos { x: 0.0, y: 0.0 };
    let mut window: PistonWindow = WindowSettings::new("Hello Piston!", [600; 2]).build().unwrap();

    let mut game = Game::new(&window);

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
            _ => {}
        }
    }
}