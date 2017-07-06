use specs::{Component, DispatcherBuilder, Dispatcher, ReadStorage, System, VecStorage, World,
            WriteStorage, Join, Fetch, HashMapStorage};

use std::collections::HashSet;
use piston_window::Button;

pub struct Delta(pub f64);
pub struct GameInput(pub HashSet<Button>);

#[derive(Debug)]
pub struct Pos {
    pub x: f64,
    pub y: f64,
}

impl Component for Pos {
    type Storage = VecStorage<Self>;
}


pub enum Bounds {
    Rectangle(f64, f64)
}

impl Component for Bounds {
    type Storage = VecStorage<Self>;
}

pub struct Player(pub i32);
impl Component for Player {
    type Storage = HashMapStorage<Self>;
}

#[derive(Clone)]
#[derive(Copy)]
pub struct Vel {
    pub x: f64,
    pub y: f64,
}

impl Component for Vel {
    type Storage = VecStorage<Self>;
}

pub struct CollisionObjectData{
}

impl Component for CollisionObjectData{
    type Storage = VecStorage<CollisionObjectData>;
}
