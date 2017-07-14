
use std::collections::HashSet;
use std::collections::HashMap;
use std::hash::Hash;
use std::cmp::min;

pub struct IdStore(HashSet<usize>, usize);

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

    pub fn release(&mut self, id: usize) {
        self.0.remove(&id);
        self.1 = min(id, self.1);
    }
    fn next_after_add(&self, added: usize) -> usize {
        let mut next = added + 1;
        while self.0.contains(&next) {
            next += 1;
        }
        next
    }
}

pub struct IdMap<T>(IdStore, HashMap<T, usize>);

impl<T> IdMap<T>
    where T: Eq + Hash + Copy 
{
    pub fn new() -> IdMap<T> {
        IdMap(IdStore::new(), HashMap::new())
    }

    pub fn get(&mut self, id: T) -> usize {
       let m = &mut self.1;
       let s = &mut self.0;
       let i = m.entry(id).or_insert_with(|| {s.get()});
       i.clone()
    }

    pub fn release(&mut self, id: T) {
        if let Some(i) = self.1.remove(&id) {
            self.0.release(i);
        }
    }
}