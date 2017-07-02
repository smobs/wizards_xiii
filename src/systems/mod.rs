extern crate specs;
extern crate piston_window;

use specs::{Component, DispatcherBuilder, Dispatcher, ReadStorage, System, VecStorage, World,
            WriteStorage, Join};
use piston_window::{rectangle, clear};

#[derive(Debug)]
pub struct Pos {
    pub x: f64,
    pub y: f64,
}

impl Component for Pos {
    type Storage = VecStorage<Self>;
}

pub struct Vel {
    pub x: f64,
    pub y: f64,
}

impl Component for Vel {
    type Storage = VecStorage<Self>;
}

pub struct UpdatePositionSystem;

impl<'a> System<'a> for UpdatePositionSystem {
    type SystemData = (WriteStorage<'a, Pos>, ReadStorage<'a, Vel>);
    fn run(&mut self, (mut pos, vel): Self::SystemData) {
        for (pos, vel) in (&mut pos, &vel).join() {
            pos.x += vel.x;
            pos.y += vel.y;
        }
    }
}
