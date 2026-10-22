use crate::dense_set::{DenseSet, ErasedDenseSet};
use std::collections::HashMap;
use std::any::TypeId;
use crate::entity::Entity;



pub struct Storage {
    sets: HashMap<TypeId, Box<dyn ErasedDenseSet>>,
    entities: Vec<Entity>,
}

impl Storage {
    pub fn new() -> Self {
        Self{ sets: HashMap::new(), entities: Vec::new()}
    }
    pub fn register<T: 'static>(&mut self) {
        self.sets.entry(TypeId::of::<T>()).or_insert_with(|| Box::new(DenseSet::<T>::new()));
    }
    
    pub fn get<T: 'static>(&self) -> Option<&DenseSet<T>> {
        self.sets.get(&TypeId::of::<T>())?.as_any().downcast_ref::<DenseSet<T>>()
    }
    
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut DenseSet<T>> {
        self.sets.get_mut(&TypeId::of::<T>())?.as_any_mut().downcast_mut::<DenseSet<T>>()
    }
    
    pub fn query_mut<T: 'static>(&mut self, f: impl Fn(Entity, &mut T) -> ()) {
        if let Some(st) = self.sets.get_mut(&TypeId::of::<T>()) {
            if let Some(dense_set) =  st.as_any_mut().downcast_mut::<DenseSet<T>>() {
                for item in dense_set {
                    if self.entities.len() <= item.id as usize {
                        continue;
                    }
                    f(self.entities[item.id as usize].clone(), &mut item.item);
                }
            }
        }
    }
    
    pub fn query<T: 'static>(&self, f: impl Fn(Entity, &T) -> ()) {
        if let Some(st) = self.sets.get(&TypeId::of::<T>()) {
            let dense_set = st.as_any().downcast_ref::<DenseSet<T>>();
            if let Some(dense_set) = dense_set {
                // 获取实体这里不太好，后续看能否优化下
                for item in dense_set {
                    if self.entities.len() <= item.id as usize {
                        continue; 
                    }
                    f(self.entities[item.id as usize].clone(), &item.item);
                }
            }
        }
    }
    
    pub fn add_entity(&mut self, entity: Entity) {
        let entity_id = entity.id as usize;
        if entity_id < self.entities.len() {
            if self.entities[entity_id].generation.get() >= entity.generation.get() {
                return;
            }
            self.entities[entity_id] = entity;
        } else {
            let new_size = entity_id + 1;
            self.entities.resize(new_size, Entity::null());
            self.entities[entity_id] = entity;
        }
    }
    
    pub fn add_component<T: 'static>(&mut self, entity: Entity, component: T) {
        if !self.is_valid_entity(&entity) {
            return; 
        }

        if let Some(st) = self.sets.get_mut(&TypeId::of::<T>()) {
            if let Some(dense_set) = st.as_any_mut().downcast_mut::<DenseSet<T>>() {
                dense_set.add(entity, component);
            }
        }
    }
    
    fn is_valid_entity(&self, entity: &Entity) -> bool {
        entity.id < self.entities.len() as u32 && self.entities[entity.id as usize].generation.get() == entity.generation.get()
    }
    
}