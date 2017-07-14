
use std::collections::HashSet;
use std::cmp::min;

struct IdStore(HashSet<usize>, usize);

impl IdStore {
    pub fn new() -> IdStore {
        IdStore(HashSet::new(), 0)
    }
    pub fn get(&mut self) -> usize {
        let id = self.1;
        self.0.insert(id);
        self.1 = self.next_after_add(id);
        id
    }

    pub fn release(&mut self, id : usize) {
        self.0.remove(&id);
        self.1 = min(id, self.1);
    }
    fn next_after_add(&self, added : usize) -> usize{
        let mut next = added+1;
            while self.0.contains(&next){
                next += 1;
            }
            next
        }
}
