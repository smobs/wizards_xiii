use specs::{Component, DispatcherBuilder, Dispatcher, ReadStorage, System, VecStorage, World,
            WriteStorage, Join, Fetch, HashMapStorage, Entities, Entity};

use ncollide::world::*;
use ncollide::shape::*;
use nalgebra as na;
use nalgebra::*;
use systems::components::*;
use std;
use std::iter::*;
use std::sync::Arc;
use ncollide::partitioning::*;
use ncollide::bounding_volume::*;
use std::collections::HashSet;
use std::collections::HashMap;
use systems::id_store::*;
pub struct CollisionSystem(CollisionWorld2<f64, Entity>);

trait UpdateableCollision {
    fn get_current_part_ids<F>(&self, &mut F) -> HashMap<usize, usize>
        where F: FnMut(usize) -> usize;
    fn get_shape_handle(&self, usize) -> Option<ShapeHandle2<f64>>;
    fn part_changed(&self, usize, &Self) -> bool;
}

impl UpdateableCollision for Bounds {
    fn get_current_part_ids<F>(&self, get: &mut F) -> HashMap<usize, usize>
        where F: FnMut(usize) -> usize
    {
        let i = get(0);
        let mut h = HashMap::new();
        h.insert(i, 0);
        h
    }

    fn get_shape_handle(&self, index: usize) -> Option<ShapeHandle2<f64>> {
        if index == 0 {
            match self {
                &Bounds::Rectangle(x, y) => Some(ShapeHandle::new(create_rectangle(x, y))),
                &Bounds::Circle(r) => Some(ShapeHandle::new(create_circle(r))),
                &Bounds::Polygon(ref ps) => Some(ShapeHandle::new(create_polygon(ps))),
                &Bounds::Grid { points: ref ps, height: h, width: w } => None,
            }
        } else {
            None
        }
    }
    fn part_changed(&self, part: usize, old: &Self) -> bool {
        if let &Bounds::Grid { points: _, height: _, width: _ } = self {
            false
        } else {
            part != 0 && *self != *old
        }
    }
}
impl CollisionSystem {
    pub fn new() -> Self {
        let world = CollisionWorld::new(0.02, false);
        CollisionSystem(world)
    }
}

fn create_circle(r: f64) -> Ball2<f64> {
    Ball::new(r)
}
fn create_rectangle(x: f64, y: f64) -> Cuboid2<f64> {
    Cuboid::new(Vector2::new(x / 2.0, y / 2.0))
}
fn create_polygon(ps: &Box<Vec<[f64; 2]>>) -> Polyline2<f64> {
    let max_index = ps.len();
    let points = Vec::from_iter(ps[..].into_iter().map(|p| Point2::new(p[0], p[1])));
    let indicies = Vec::from_iter((1..(max_index))
        .map(|p| Point2::new(p - 1, p))
        .chain(once(Point2::new(max_index - 1, 0))));
    Polyline::new(Arc::new(points), Arc::new(indicies), None, None)
}


impl<'a> CollisionSystem {
    fn remove_changed(&mut self,
                      ent: &Entities<'a>,
                      col: &mut WriteStorage<'a, CollisionObjectData>,
                      bounds: &ReadStorage<'a, Bounds>) {
        let world = &mut self.0;
        for (ent, col, bounds) in (&**ent, col, bounds).join() {
            let id = ent.id() as usize;
            for (&id, &part) in bounds.get_current_part_ids(&mut |x| id).iter() {
                match &col.current_bounds {
                    &Some(ref b) => {
                        if bounds.part_changed(part, b) {
                            println!("Removing entity {:?}", id);
                            world.deferred_remove(id);
                        }
                    }
                    _ => {}
                }
            }
        }
        world.update();
    }
    fn update_collisions(&mut self,
                         ent: &Entities<'a>,
                         pos: &WriteStorage<'a, Pos>,
                         col: &mut WriteStorage<'a, CollisionObjectData>,
                         bounds: &ReadStorage<'a, Bounds>) {
        let world = &mut self.0;
        for (ent, pos, col, bounds) in (&**ent, pos, col, bounds).join() {
            let id = ent.id() as usize;
            for (&id, &part) in bounds.get_current_part_ids(&mut |x| id).iter() {
                let p = Isometry2::new(Vector2::new(pos.x, pos.y), na::zero());
                if let Some(_) = world.collision_object(id) {
                    world.deferred_set_position(id, p)
                } else {
                    if let Some(shape) = bounds.get_shape_handle(part) {
                        let mut cg = CollisionGroups::new();
                        world.deferred_add(id, p, shape, cg, GeometricQueryType::Contacts(0.0), ent);
                        let b = (*bounds).clone();
                        col.current_bounds = Some(b);
                    }
                }
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
                if (!p1.is_empty()) {
                    println!("e1 contacts {:?}", p1);
                }
                col.contacts.insert(e2.data, p1);
            }
            if let Some(col) = col.get_mut(e2.data) {
                if !p2.is_empty() {
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
        self.remove_changed(&ent, &mut col, &bounds);
        while i < 10 && dirty {
            {
                self.update_collisions(&ent, &pos, &mut col, &bounds);
            }
            {
                dirty = self.basic_physics(&mut pos, &vel);
            }
            {
                if (i == 0) {
                    self.update_collision_objects(&mut col)
                }
            }
            i += 1;
        }
        {
        }

    }
}
