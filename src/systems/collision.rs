
extern crate nalgebra;

use specs::{Component, DispatcherBuilder, Dispatcher, ReadStorage, System, VecStorage, World,
            WriteStorage, Join, Fetch, HashMapStorage, Entities, Entity};

use ncollide::world::*;
use ncollide::shape::*;
use self::nalgebra as na;
use self::nalgebra::{Vector2, Isometry2};
use systems::components::*;
use std;

pub struct CollisionSystem(CollisionWorld2<f64, Entity>);

impl CollisionSystem {
    pub fn new() -> Self {
        let world = CollisionWorld::new(0.02, false);
        CollisionSystem(world)
    }
}

impl<'a> System<'a> for CollisionSystem {
    type SystemData = (Entities<'a>,
     WriteStorage<'a, Pos>,
     ReadStorage<'a, CollisionObjectData>,
     ReadStorage<'a, Bounds>);
    fn run(&mut self, (ent, mut pos, col, bounds): Self::SystemData) {
        let world = &mut self.0;
        for (ent, pos, _, bounds) in (&*ent, &mut pos, &col, &bounds).join() {
            let id = ent.id() as usize;
            let p = Isometry2::new(Vector2::new(pos.x, pos.y), na::zero());
            if let Some(_) = world.collision_object(id) {
                world.deferred_set_position(id, p)
            } else {
                let shape = match *bounds {
                    Bounds::Rectangle(x, y) => {
                        ShapeHandle::new(Cuboid::new(Vector2::new(x / 2.0, y / 2.0)))
                    }
                    Bounds::Circle(r) => ShapeHandle::new(Ball::new(r)),
                };
                world.deferred_add(id,
                                   p,
                                   shape,
                                   CollisionGroups::new(),
                                   GeometricQueryType::Contacts(0.0),
                                   ent);
            }
        }
        world.update();
        for (e1, e2, ca) in world.contact_pairs() {
            let mut contacts = std::vec::Vec::new();
            ca.contacts(&mut contacts);
            for contact in contacts {
                let mut move_vec = contact.normal * contact.depth * -0.5;
                {
                    if let Some(p1) = pos.get_mut(e1.data){
                        p1.x += move_vec.x;
                        p1.y += move_vec.y;
                    }
                }
                move_vec *= -1.0;
                {
                    if let Some(p2) = pos.get_mut(e2.data){
                        p2.x += move_vec.x;
                        p2.y += move_vec.y;
                    }
                }
                
            }
        }
    }
}