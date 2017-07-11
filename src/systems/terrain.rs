use specs::{ReadStorage, System, VecStorage, World, WriteStorage, Join, Fetch, HashMapStorage};

use systems::components::*;
use std::collections::HashSet;
use std::iter::*;
use nalgebra::*;
pub struct TerrainSystem;


fn get_edges(point: &Vector2<isize>, points: &HashSet<[isize; 2]>) -> HashSet<[isize; 2]> {
    let mut neighboughs = HashSet::new();
    for p1 in point[0] - 1..point[0] + 2 {
        for p2 in point[1] - 1..point[1] + 2 {
            if !(point[0] == p1 && point[1] == p2) {
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
    let mut direction = Vector2::new(-1, 0);
    if let Some(mut p) = points.iter().next().cloned() {
        let mut current_point = Vector2::new(p[0], p[1]);
        let mut edges;
        while !complete {
            while start == None {
                edges = get_edges(&current_point, &points);
                let edge = !edges.is_empty();
                if edge {
                    start = Some(current_point);
                } else {
                    current_point[0] -= 1;
                }
            }
            let x = current_point[0] as f64;
            let y = current_point[1] as f64;
            poly.push([x, y]);
            {
                let clockwise: [(isize, isize); 8] = [(-1, 1), (0, 1), (1, 1), (1, 0), (1, -1),
                                                      (0, -1), (-1, -1), (-1, 0)];

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
    let mut vec = vec!();
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
                        if (tb[0] == ob[0] && ob[0] == p[0]) || (tb[1] == ob[1] && ob[1] == p[1]) {
                            one_back = Some(p);
                        } else {
                            vec.push(tb);
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
impl<'a> System<'a> for TerrainSystem {
    type SystemData = (WriteStorage<'a, Terrain>, WriteStorage<'a, Bounds>);
    fn run(&mut self, (mut terrain, mut bounds): Self::SystemData) {
        for (mut terrain, mut bounds) in (&mut terrain, &mut bounds).join() {
            if terrain.dirty {
                *bounds = new_bounds(&terrain.points);
                (*terrain).dirty = false;
            }
        }
    }
}