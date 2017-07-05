use specs::{Component, DispatcherBuilder, Dispatcher, ReadStorage, System, VecStorage, World,
            WriteStorage, Join, Fetch, HashMapStorage};
use piston_window::{rectangle, clear};
use piston_window::Button::Keyboard;
use piston_window::Button;
use piston_window::Key::*;

use systems::components::*;

use std::collections::HashSet;

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

pub struct UpdateControlSystem;
fn get_vel(pid: i32, buttons: &HashSet<Button>) -> Vel {
    let mut vel = Vel { x: 0.0, y: 0.0 };
    let (up, down, left, right) = match pid {
        2 => (Up, Down, Left, Right),
        _ => (W, S, A, D),
    };
    if buttons.contains(&Keyboard(up)) {
        vel.y = -1.0;
    }
    if buttons.contains(&Keyboard(down)) {
        vel.y = 1.0;
    }
    if buttons.contains(&Keyboard(left)) {
        vel.x = -1.0;
    }
    if buttons.contains(&Keyboard(right)) {
        vel.x = 1.0;
    }
    vel
}
impl<'a> System<'a> for UpdateControlSystem {
    type SystemData = (ReadStorage<'a, Player>, WriteStorage<'a, Vel>, Fetch<'a, GameInput>);
    fn run(&mut self, (player, mut vel, gi): Self::SystemData) {
        for (p, mut vel) in (&player, &mut vel).join() {
            *vel = get_vel(p.0, &gi.0);
        }
    }
}