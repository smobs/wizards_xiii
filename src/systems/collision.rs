
extern crate nalgebra;

use specs::{Component, DispatcherBuilder, Dispatcher, ReadStorage, System, VecStorage, World,
            WriteStorage, Join, Fetch, HashMapStorage};

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
    type SystemData = (WriteStorage<'a, Pos>, ReadStorage<'a, CollisionObjectData>);
    fn run(&mut self, (mut pos, col): Self::SystemData) {
        let world = &mut self.0;
        for (pos, col) in (&mut pos, &col).join() {

            let p = Isometry2::new(Vector2::new(pos.x, pos.y), nalgebra::zero());
            if let Some(_) = world.collision_object(col.id) {
                world.deferred_set_position(col.id, p)
            } else {
                let cuboid = ShapeHandle::new(Cuboid::new(Vector2::new(50.0, 50.0)));
                world.deferred_add(col.id,
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