use specs::{Component, DispatcherBuilder, Dispatcher, ReadStorage, System, VecStorage, World,
            WriteStorage, Join, Fetch, HashMapStorage};

use std::collections::HashSet;
use piston_window::Button;
use std::boxed;

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
    Rectangle(f64, f64),
    Circle(f64),
    Polygon(Box<Vec<[f64; 2]>>),
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
    pub group_id : usize,
}

impl Component for CollisionObjectData{
    type Storage = VecStorage<CollisionObjectData>;
}

pub struct Terrain{
    pub dirty : bool,
    pub points : HashSet<[isize; 2]>
}

impl Terrain {
    pub fn new(x : usize, y : usize, width : usize, height : usize) -> Terrain{
        let mut ps = HashSet::new();
        for x in x..(x+width){
            for y in (500 - (x as isize)) .. 500{
                ps.insert([x as isize,y as isize]);
            }
        }
        Terrain{dirty: true, points: ps}
    }
}

impl Component for Terrain {
    type Storage = HashMapStorage<Terrain>;
}