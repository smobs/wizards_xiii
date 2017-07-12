use specs::{ReadStorage, System, VecStorage, World, WriteStorage, Join, Fetch, HashMapStorage};

use systems::components::*;
use std::collections::HashSet;
use std::iter::*;
use nalgebra::*;
pub struct TerrainSystem;


fn get_edges(point: &Vector2<f64>, points: &HashSet<[isize; 2]>) -> HashSet<[isize; 2]> {
    let mut neighboughs = HashSet::new();
    let px = point[0] as isize;
    let py = point[1] as isize;
    for p1 in px - 1..px + 2 {
        for p2 in py - 1..py + 2 {
            if !(px == p1 && py == p2) {
                neighboughs.insert([p1, p2]);
            }
        }
    }
    neighboughs.difference(points).cloned().collect()
}
fn new_bounds(points: &HashSet<[isize; 2]>) -> Bounds {
    let mut poly = vec![];
    let mut complete = false;
    let mut start = None;
    let mut direction = Vector2::new(-1.0, 0.0);
    if let Some(mut p) = points.iter().next().cloned() {
        let mut current_point = Vector2::new(p[0] as f64, p[1] as f64);
        let mut edges;
        while !complete {
            while start == None {
                edges = get_edges(&current_point, &points);
                let edge = !edges.is_empty();
                if edge {
                    start = Some(current_point);
                } else {
                    current_point[0] -= 1.0;
                }
            }
            poly.push(current_point);
            {
                let clockwise: [(f64, f64); 8] = [(-1.0, 1.0),
                                                  (0.0, 1.0),
                                                  (1.0, 1.0),
                                                  (1.0, 0.0),
                                                  (1.0, -1.0),
                                                  (0.0, -1.0),
                                                  (-1.0, -1.0),
                                                  (-1.0, 0.0)];

                let count = clockwise.iter()
                    .take_while(|&&(x, y)| x != direction[0] || y != direction[1])
                    .count();
                for v in clockwise.iter()
                    .cycle()
                    .skip((count + 7) % 8)
                    .map(|&(x, y)| Vector2::new(x, y)) {
                    let mut at = current_point.clone();
                    at += v;
                    edges = get_edges(&at, &points);
                    if !edges.is_empty() {
                        current_point = at;
                        direction = v;
                        break;
                    }
                }
            }
            if start == Some(current_point) {
                complete = true;
            }
        }
    }
    let mut two_back = None;
    let mut one_back = None;
    let mut vec = vec![];
    for p in poly {
        match two_back {
            None => {
                two_back = Some(p);
            }
            Some(tb) => {
                match one_back {
                    None => {
                        one_back = Some(p);
                    }
                    Some(ob) => {
                        let d1 = ob - tb;
                        let d2 = p - ob;

                        if d2.normalize() == d1.normalize() {
                            one_back = Some(p);
                        } else {
                            vec.push([tb[0] as f64, tb[1] as f64]);
                            two_back = Some(ob);
                            one_back = Some(p);
                        }
                    }
                }
            }
        }

    }
    return Bounds::Polygon(Box::new(vec));
}

fn handle_collision(terrain: &mut Terrain, col: &CollisionObjectData) {

    for contact in col.contacts.values().flat_map(|x| x) {
        terrain.dirty = true;
        for x in -5..5 {
            for y in -5..5 {
                let p = [(contact[0]) as isize + x, (contact[1]) as isize + y];
                !terrain.points.remove(&p); 
            }
        }
    }
}
impl<'a> System<'a> for TerrainSystem {
    type SystemData = (WriteStorage<'a, Terrain>,
     WriteStorage<'a, Bounds>,
     ReadStorage<'a, CollisionObjectData>);
    fn run(&mut self, (mut terrain, mut bounds, col): Self::SystemData) {
        for (mut terrain, mut bounds, col) in (&mut terrain, &mut bounds, &col).join() {
            handle_collision(terrain, col);
            if terrain.dirty {
                *bounds = new_bounds(&terrain.points);
                (*terrain).dirty = false;
            }
        }
    }
}