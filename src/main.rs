#![feature(test)]
extern crate test;
extern crate scoped_threadpool;

use std::vec::Vec;
use std::collections::HashMap;
use std::any::{Any, TypeId};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Entity(u64);

#[derive(Clone, Copy)]
struct Position {
    x: u32,
    y: u32,
    z: u32,
}

struct World {
    next_entity_id: u64,
    entities: Vec<Entity>,
    components: HashMap<TypeId, HashMap<Entity, Box<Any>>>,
}

impl World {
    fn new() -> Self {
        World {
            next_entity_id: 0,
            entities: Vec::new(),
            components: HashMap::new(),
        }
    }

    fn create_entity(&mut self) -> Entity {
        let e = Entity(self.next_entity_id);
        self.next_entity_id += 1;
        self.entities.push(e);
        e
    }

    fn ensure_components(&mut self, t: TypeId) -> &mut HashMap<Entity, Box<Any>> {
        let m = &mut self.components;

        if m.contains_key(&t) {
            return m.get_mut(&t).unwrap();
        }

        let cs = HashMap::new();
        m.insert(t, cs);
        m.get_mut(&t).unwrap()
    }

    fn add_component<T: Any + Clone>(&mut self, e: Entity, c: T) {
        let t = TypeId::of::<T>();
        let cs = self.ensure_components(t);
        cs.insert(e, Box::new(c));
    }

    fn component_mut<T: Any + Clone>(&mut self, e: &Entity) -> Option<&mut T> {
        let t = TypeId::of::<T>();
        if let Some(cs) = self.components.get_mut(&t) {
            match cs.get_mut(e) {
                Some(c) => c.downcast_mut::<T>(),
                _ => None,
            }
        } else {
            None
        }
    }

    fn components<T: Any + Clone>(&self, es: &[Entity]) -> Vec<T> {
        let t = TypeId::of::<T>();
        if let Some(cs) = self.components.get(&t) {
            es.iter().filter_map(|e| {
                cs.get(e).map(|c| (*(**c).downcast_ref::<T>().unwrap()).clone())
            }).collect()
        } else {
            Vec::new()
        }
    }
}

struct Task<T: Any + Clone> {
    cs: Vec<T>,
}

fn main() {
}

const ENTITY_COUNT: usize = 100000;

#[bench]
fn bench(b: &mut test::Bencher) {
    let mut w = World::new();

    for _ in 0..ENTITY_COUNT {
        let e = w.create_entity();
        w.add_component(e, Position { x: 0, y: 0, z: 0 });
    }

    let thread_count = 4;
    let chunks = 128;
    let mut pool = scoped_threadpool::Pool::new(thread_count);

    b.iter(|| {
        pool.scoped(|scope|{
            for es in w.entities.chunks(chunks) {
                let t = Task { cs: w.components::<Position>(es) };
                scope.execute(move || {
                    for ref mut c in t.cs {
                        c.x += 1;
                    }
                });
            }
        });
    });
}

#[bench]
fn bench_single_thread(b: &mut test::Bencher) {
    let mut w = World::new();

    for _ in 0..ENTITY_COUNT {
        let e = w.create_entity();
        w.add_component(e, Position { x: 0, y: 0, z: 0 });
    }

    b.iter(|| {
        for e in w.entities.clone() {
            let c = w.component_mut::<Position>(&e).unwrap();
            c.x += 1;
        }
    });
}

