
extern crate nalgebra;

use specs::{Component, DispatcherBuilder, Dispatcher, ReadStorage, System, VecStorage, World,
            WriteStorage, Join, Fetch, HashMapStorage, Entities};

use ncollide::world::*;
use ncollide::shape::*;
use self::nalgebra as na;
use self::nalgebra::{Vector2, Isometry2};
use systems::components::*;

pub struct CollisionSystem(CollisionWorld2<f64, ()>);

impl CollisionSystem {
    pub fn new() -> Self {
        let world = CollisionWorld::new(0.02, false);
        CollisionSystem(world)
    }
}

impl<'a> System<'a> for CollisionSystem {
    type SystemData = (Entities<'a>, WriteStorage<'a, Pos>, ReadStorage<'a, CollisionObjectData>);
    fn run(&mut self, (ent, mut pos, col): Self::SystemData) {
        let world = &mut self.0;
        for (ent, pos, col) in (&*ent, &mut pos, &col).join() {
            let id = ent.id() as usize;
            let p = Isometry2::new(Vector2::new(pos.x, pos.y), na::zero());
            if let Some(_) = world.collision_object(id) {
                world.deferred_set_position(id, p)
            } else {
                let cuboid = ShapeHandle::new(Cuboid::new(Vector2::new(50.0, 50.0)));
                world.deferred_add(id,
                                   p,
                                   cuboid,
                                   CollisionGroups::new(),
                                   GeometricQueryType::Contacts(0.0),
                                   ());
            }
            world.update();
            for (e1, e2, _) in world.contact_pairs(){
                println!("{:?} hit {:?}", e1.uid, e2.uid);
            }
        }
    }
}