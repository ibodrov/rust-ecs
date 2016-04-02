#![feature(test)]
extern crate test;

use std::vec::Vec;
use std::collections::HashMap;
use std::any::{Any, TypeId};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Entity(u64);

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

    fn add_component<T: Any>(&mut self, e: Entity, c: T) {
        let t = TypeId::of::<T>();
        let cs = self.ensure_components(t);
        cs.insert(e, Box::new(c));
    }

    fn component_mut<T: Any>(&mut self, e: &Entity) -> Option<&mut T> {
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
}

fn main() {
}

#[bench]
fn bench(b: &mut test::Bencher) {
    let mut w = World::new();

    for _ in 0..100000 {
        let e = w.create_entity();
        w.add_component(e, Position { x: 0, y: 0, z: 0 });
    }

    b.iter(|| {
        for e in w.entities.clone() {
            let p = w.component_mut::<Position>(&e).unwrap();
            p.x += 1;
        }
    });
}
