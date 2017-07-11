use specs::{Component, DispatcherBuilder, Dispatcher, ReadStorage, System, VecStorage, World,
            WriteStorage, Join, Fetch, HashMapStorage};


pub struct TerrainSystem;

impl<'a> System<'a> for TerrainSystem {
    type SystemData = (); 
    fn run(&mut self, data: Self::SystemData){
        
    } 
}