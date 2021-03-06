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
pub struct CollisionSystem(CollisionWorld2<f64, Entity>, IdMap<(usize, usize)>);

trait UpdateableCollision {
    fn get_current_part_ids<F>(&self, &mut F) -> HashMap<usize, usize>
        where F: FnMut(usize) -> usize;
    fn get_shape_handle(&self, usize) -> Option<ShapeHandle2<f64>>;
    fn get_position_for_part(&self, usize) -> Option<[usize; 2]>;
    fn parts_changed(&self, &Self) -> HashSet<usize>;
}
fn get_point_grid_id(w: usize, h: usize, x: usize, y: usize) -> usize {
    y * w + x
}

fn get_point_from_id(ps: &HashSet<[usize; 2]>,
                     w: usize,
                     h: usize,
                     id: usize)
                     -> Option<[usize; 2]> {
    if w * h > id {
        let p = [id % w, id / w];
        if ps.contains(&p) { Some(p) } else { None }
    } else {
        None
    }
}


impl UpdateableCollision for Bounds {
    fn get_current_part_ids<F>(&self, get: &mut F) -> HashMap<usize, usize>
        where F: FnMut(usize) -> usize
    {
        let mut map = HashMap::new();
        let i = get(0);
        map.insert(i, 0);

        map
    }

    fn get_shape_handle(&self, index: usize) -> Option<ShapeHandle2<f64>> {
        match self {
            &Bounds::Rectangle(x, y) => Some(ShapeHandle::new(create_rectangle(x, y))),
            &Bounds::Circle(r) => Some(ShapeHandle::new(create_circle(r))),
            &Bounds::Polygon(ref ps) => Some(ShapeHandle::new(create_polygon(ps))),
        }
    }

    fn get_position_for_part(&self, index: usize) -> Option<[usize; 2]> {

        Some([0, 0])

    }
    fn parts_changed(&self, old: &Self) -> HashSet<usize> {

        let mut h = HashSet::new();
        if *self != *old {
            h.insert(0);
        }
        h
    }
}
impl CollisionSystem {
    pub fn new() -> Self {
        let world = CollisionWorld::new(0.02, false);
        CollisionSystem(world, IdMap::new())
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
        let idmap = &mut self.1;
        for (ent, col, bounds) in (&**ent, col, bounds).join() {
            let eid = ent.id() as usize;
            match &col.current_bounds {
                &Some(ref b) => {
                    for p in bounds.parts_changed(b) {
                        let id = idmap.get((eid, p));
                        println!("Removing entity {:?}", id);
                        world.deferred_remove(id);
                        idmap.release((eid, p));
                    }
                }
                _ => {}
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
        let idmap = &mut self.1;
        for (ent, pos, col, bounds) in (&**ent, pos, col, bounds).join() {
            let eid = ent.id() as usize;
            for (&id, &part) in bounds.get_current_part_ids(&mut |x| idmap.get((eid, x))).iter() {
                if let Some(p) = bounds.get_position_for_part(part) {
                    {
                        let p = Isometry2::new(Vector2::new(pos.x + (p[0] as f64),
                                                            pos.y + (p[1] as f64)),
                                               na::zero());
                        if let Some(_) = world.collision_object(id) {
                            world.deferred_set_position(id, p)
                        } else {
                            if let Some(shape) = bounds.get_shape_handle(part) {
                                let mut cg = CollisionGroups::new();
                                cg.set_membership(&[col.group_id]);
                                cg.set_blacklist(&[col.group_id]);
                                world.deferred_add(id,
                                                   p,
                                                   shape,
                                                   cg,
                                                   GeometricQueryType::Contacts(0.0),
                                                   ent);
                                let b = (*bounds).clone();
                                col.current_bounds = Some(b);
                            }
                        }
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
                if !p1.is_empty() {
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
                if i == 0 {
                    self.update_collision_objects(&mut col)
                }
            }
            i += 1;
        }
        {
        }

    }
}
