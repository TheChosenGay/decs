use std::collections::VecDeque;
use std::cell::Cell;
use std::fmt::Display;

use serde::{Deserialize, Serialize};
use bincode::{serialize, deserialize};


pub const NULL_ENTITY: Entity = Entity { id: 0, generation: Cell::new(0) };

// generation from zero, id from one
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entity {
    pub(crate) id: u32,
    pub(crate) generation: Cell<u32>,
}

impl Entity {
    pub fn null() -> Self {
        NULL_ENTITY
    }
    
    pub fn is_null(&self) -> bool {
        self.id == 0
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serialize(self).unwrap()
    } 
    
    pub fn from_bytes(bytes: &[u8]) -> Self {
        deserialize(bytes).unwrap()
    }
}

impl Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity {{ id: {}, generation: {} }}", self.id, self.generation.get())
    }
}

// 1. 生成 2. 回收 
pub struct EntitySpawner {
    avaliable_entities: VecDeque<Entity>,
    entity_id: u32,
}

impl EntitySpawner {
    pub fn new() -> Self {
        Self { avaliable_entities: VecDeque::new(), entity_id: 0 }
    }
    
    pub fn spawn_entity(&mut self) -> Entity {
        if let Some(entity) = self.avaliable_entities.pop_front() {
            return entity;
        }

        self.generate_entity()
    }
    
    pub fn recycle_entity(&mut self, entity: Entity) {
        entity.generation.set(entity.generation.get() + 1);
        self.avaliable_entities.push_back(entity);
    }
    
    fn generate_entity(&mut self) -> Entity {
        self.entity_id += 1;
        self.avaliable_entities.push_back(Entity {
            id: self.entity_id,
            generation: Cell::new(0),
        });
        self.avaliable_entities.pop_front().unwrap()
    }
}
