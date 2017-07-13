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

pub struct CollisionSystem(CollisionWorld2<f64, Entity>);

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

struct GridShape {
    bvt: BVT<usize, AABB2<f64>>,
    pointMap: HashMap<usize, [usize; 2]>,
    points: HashSet<[usize; 2]>,
    height: usize,
    width: usize,
}
impl GridShape {
    pub fn new(ps: &HashSet<[usize; 2]>, width: usize, height: usize) -> GridShape {
        let map: HashMap<usize, [usize; 2]> = (0..).zip(ps.iter().map(|p| p.clone())).collect();
        let mut aabbs: Vec<(usize, AABB2<f64>)> = vec![];

        for (&i, p) in map.iter() {
            aabbs.push((i, Self::create_aabb(p[0], p[0])));
        }

        GridShape {
            bvt: BVT::new_balanced(aabbs),
            points: ps.clone(),
            pointMap: map,
            height: height,
            width: width,
        }
    }
    fn create_aabb(x: usize, y: usize) -> AABB2<f64> {
        aabb(&Cuboid::new(Vector2::new(1.0, 1.0)),
             &Isometry2::new(Vector2::new(x as f64, y as f64), na::zero()))
    }
}
impl CompositeShape<Point2<f64>, Isometry2<f64>> for GridShape {
    fn map_part_at(&self, i: usize, f: &mut FnMut(&Isometry2<f64>, &Shape2<f64>)) {
        // The translation needed to center the cuboid at the point (1, 1).
        if let Some(p) = self.pointMap.get(&i) {

            let transform = Isometry2::new(Vector2::new(p[0] as f64, p[1] as f64), na::zero());

            // Create the cuboid on-the-fly.
            let cuboid = Cuboid2::new(Vector2::new(1.0, 1.0));

            // Call the function.
            f(&transform, &cuboid)
        }
    }

    fn map_transformed_part_at(&self,
                               i: usize,
                               m: &Isometry2<f64>,
                               f: &mut FnMut(&Isometry2<f64>, &Shape2<f64>)) {
        // Prepend the translation needed to center the cuboid at the point (1, 1).
        if let Some(p) = self.pointMap.get(&i) {

        let transform = m * Translation2::new(p[0] as f64, p[1] as f64);

        // Create the cuboid on-the-fly.
        let cuboid = Cuboid2::new(Vector2::new(1.0, 1.0));

        // Call the function.
        f(&transform, &cuboid)
        }
    }

    fn aabb_at(&self, i: usize) -> AABB2<f64> {
        // Compute the i-th AABB.
        match self.pointMap.get(&i) {
            Some(p) => Self::create_aabb(p[0], p[0]), 
            _ => panic!("Aaah"),
        }
    }

    fn bvt(&self) -> &BVT<usize, AABB2<f64>> {
        // Reference to the acceleration structure.
        &self.bvt
    }
}
impl Shape<Point2<f64>, Isometry2<f64>> for GridShape {
    fn aabb(&self, m: &Isometry2<f64>) -> AABB2<f64> {
        AABB2::new(m.translation * Point2::new(-1.0, -1.0),
                   m.translation * Point2::new(self.width as f64, self.height as f64))
    }

    fn as_composite_shape(&self) -> Option<&CompositeShape2<f64>> {
        Some(self)
    }
}

fn shape_from_bounds(bounds: &Bounds) -> ShapeHandle2<f64> {
    match bounds {
        &Bounds::Rectangle(x, y) => ShapeHandle::new(create_rectangle(x, y)),
        &Bounds::Circle(r) => ShapeHandle::new(create_circle(r)),
        &Bounds::Polygon(ref ps) => ShapeHandle::new(create_polygon(ps)),
        &Bounds::Grid { points: ref ps, height: h, width: w } => {
            ShapeHandle::new(GridShape::new(ps, w, h))
        }
    }
}
impl<'a> CollisionSystem {
    fn remove_changed(&mut self,
                      ent: &Entities<'a>,
                      col: &mut WriteStorage<'a, CollisionObjectData>,
                      bounds: &ReadStorage<'a, Bounds>) {
        let world = &mut self.0;
        for (ent, col, bounds) in (&**ent, col, bounds).join() {
            let id = ent.id() as usize;
            match &col.current_bounds {
                &Some(ref b) => {
                    if *b != *bounds {
                        println!("Removing entity {:?}", id);
                        world.deferred_remove(id);
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
        for (ent, pos, col, bounds) in (&**ent, pos, col, bounds).join() {
            let id = ent.id() as usize;
            let p = Isometry2::new(Vector2::new(pos.x, pos.y), na::zero());
            if let Some(_) = world.collision_object(id) {
                world.deferred_set_position(id, p)
            } else {
                let shape = shape_from_bounds(bounds);
                let mut cg = CollisionGroups::new();
                world.deferred_add(id, p, shape, cg, GeometricQueryType::Contacts(0.0), ent);
                let b = (*bounds).clone();
                col.current_bounds = Some(b);
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
