use specs::{Component, DispatcherBuilder, Dispatcher, ReadStorage, System, VecStorage, World,
            WriteStorage, Join, Fetch, HashMapStorage, Entities, Entity};

use ncollide::world::*;
use ncollide::shape::*;
use nalgebra as na;
use nalgebra::{Vector2, Isometry2, Point2};
use systems::components::*;
use std;
use std::iter::*;
use std::sync::Arc;

pub struct CollisionSystem(CollisionWorld2<f64, Entity>);

impl CollisionSystem {
    pub fn new() -> Self {
        let world = CollisionWorld::new(0.02, false);
        CollisionSystem(world)
    }
}

impl<'a> CollisionSystem {
    fn update_collisions(&mut self,
                         ent: &Entities<'a>,
                         pos: &WriteStorage<'a, Pos>,
                         col: &WriteStorage<'a, CollisionObjectData>,
                         bounds: &ReadStorage<'a, Bounds>) {
        let world = &mut self.0;
        for (ent, pos, col, bounds) in (&**ent, &*pos, &*col, &*bounds).join() {
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
                    Bounds::Polygon(ref ps) => {
                        let max_index = ps.len();
                        let points =
                            Vec::from_iter(ps[..].into_iter().map(|p| Point2::new(p[0], p[1])));
                        let indicies = Vec::from_iter((1..(max_index))
                            .map(|p| Point2::new(p - 1, p))
                            .chain(once(Point2::new(max_index - 1, 0))));
                        ShapeHandle::new(Polyline::new(Arc::new(points),
                                                       Arc::new(indicies),
                                                       None,
                                                       None))
                    }
                };
                let mut cg = CollisionGroups::new();
                world.deferred_add(id, p, shape, cg, GeometricQueryType::Contacts(0.0), ent);
            }
        }
        world.update();
    }

    fn basic_physics(&mut self,
                     pos: &mut WriteStorage<'a, Pos>,
                     vel: &ReadStorage<'a, Vel>)
                     -> bool {
        let world = &mut self.0;
        let mut dirty = false;
        for (e1, e2, ca) in world.contact_pairs() {
            let mut contacts = std::vec::Vec::new();
            ca.contacts(&mut contacts);
            for contact in contacts {
                let mut move_vec = contact.normal * contact.depth * -0.5;
                {
                    if let Some(p1) = pos.get_mut(e1.data) {
                        if let Some(_) = vel.get(e1.data) {
                            dirty = true;
                            p1.x += move_vec.x;
                            p1.y += move_vec.y;
                        }
                    }
                }
                move_vec *= -1.0;
                {
                    if let Some(p2) = pos.get_mut(e2.data) {
                        if let Some(_) = vel.get(e2.data) {
                            dirty = true;
                            p2.x += move_vec.x;
                            p2.y += move_vec.y;
                        }
                    }
                }
            }
        }
        return dirty;

    }
    fn clear_collision_objects(col: &mut WriteStorage<'a, CollisionObjectData>) {
        for mut col in col.join() {
            col.contacts.clear();
        }
    }
    fn update_collision_objects(&mut self, col: &mut WriteStorage<'a, CollisionObjectData>) {
        let world = &mut self.0;

        for (e1, e2, ca) in world.contact_pairs() {
            let mut contacts = std::vec::Vec::new();
            ca.contacts(&mut contacts);
            let mut p1 = vec![];
            let mut p2 = vec![];
            for contact in contacts {
                p1.push([contact.world1[0], contact.world1[1]]);
                p2.push([contact.world2[0], contact.world2[1]]);
            }
            if let Some(col) = col.get_mut(e1.data) {
                if (!p1.is_empty()){
                println!("e1 contacts {:?}", p1);
                }col.contacts.insert(e2.data, p1); 
            }
            if let Some(col) = col.get_mut(e2.data) {
                if !p2.is_empty(){
                println!("e2 contacts {:?}", p2);
                }
                col.contacts.insert(e1.data, p2);
            }
        }
    }
}

impl<'a> System<'a> for CollisionSystem {
    type SystemData = (Entities<'a>,
     WriteStorage<'a, Pos>,
     WriteStorage<'a, CollisionObjectData>,
     ReadStorage<'a, Bounds>,
     ReadStorage<'a, Vel>);
    fn run(&mut self, (ent, mut pos, mut col, bounds, vel): Self::SystemData) {
        let mut dirty = true;
        let mut i = 0;

        Self::clear_collision_objects(&mut col);
        while i < 10 && dirty {
            {
                self.update_collisions(&ent, &pos, &col, &bounds);
            }
            {
                dirty = self.basic_physics(&mut pos, &vel);
            }
            {
                if(i == 0){
                self.update_collision_objects(&mut col)
            }}
            i += 1;
        }
        {
        }

    }
}
