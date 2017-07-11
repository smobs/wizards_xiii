use specs::{ReadStorage, System, VecStorage, World,
            WriteStorage, Join, Fetch, HashMapStorage};

use systems::components::*;
use std::collections::HashSet;
use std::iter::*;
pub struct TerrainSystem;


fn is_edge(point: &[usize; 2], points: &HashSet<[usize;2]> ) -> bool {
    let mut neighboughs = HashSet::new();
    for p1 in point[0] -1 .. point[0] +2{
        for p2 in point[1] -1 .. point[1] +2{
            if !(point[0] == p1 && point[1] == p2){
                neighboughs.insert([p1,p2]);   
            }
        }   
    }
    neighboughs.is_subset(points)
}
fn new_bounds(points: &HashSet<[usize; 2]>)-> Bounds {
    let mut poly = vec!();
    let mut visited = HashSet::new();
    let mut complete = false;
    let mut start = None;
    if let Some(mut currentPoint) = points.iter().next(){
        while !complete {
            let edge = is_edge(currentPoint, &points);
            if None == start && edge {
                start = Some(currentPoint);
            }
            if edge {
                let x = currentPoint[0] as f64;
                let y = currentPoint[1] as f64;
                poly.push([x, y]);
            }
            visited.insert(currentPoint);
            if Some(currentPoint) == start{
                complete = true;
            }
        } 
    }
    return Bounds::Polygon(Box::new(poly));
} 
impl<'a> System<'a> for TerrainSystem {
    type SystemData = (WriteStorage<'a, Terrain>, WriteStorage<'a, Bounds>); 
    fn run(&mut self, (mut terrain, mut bounds): Self::SystemData){
        for (mut terrain, mut bounds) in (&mut terrain, &mut bounds).join(){
            if terrain.dirty{
                *bounds = new_bounds(&terrain.points);
                (*terrain).dirty = false;
            }
        }
    } 
}