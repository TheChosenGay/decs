use crate::dense_set::{DenseSet, ErasedDenseSet};
use std::collections::HashMap;
use std::any::TypeId;
use crate::entity::Entity;


pub trait Component {

}

pub struct Storage {
    sets: HashMap<TypeId, Box<dyn ErasedDenseSet>>,
    entities: Vec<Option<Entity>>,
}

impl Storage {
    pub fn new() -> Self {
        Self{ sets: HashMap::new(), entities: Vec::new()}
    }
    pub fn register<T: Component + 'static>(&mut self) {
        self.sets.entry(TypeId::of::<T>()).or_insert_with(|| Box::new(DenseSet::<T>::new()));
    }
    
    pub fn get<T: Component + 'static>(&self) -> Option<&DenseSet<T>> {
        self.sets.get(&TypeId::of::<T>())?.as_any().downcast_ref::<DenseSet<T>>()
    }
    
    pub fn get_mut<T: Component + 'static>(&mut self) -> Option<&mut DenseSet<T>> {
        self.sets.get_mut(&TypeId::of::<T>())?.as_any_mut().downcast_mut::<DenseSet<T>>()
    }
    
    pub fn query_mut<T: Component + 'static>(&mut self, mut f: impl FnMut(Entity, &mut T) -> ()) {
        if let Some(st) = self.sets.get_mut(&TypeId::of::<T>()) {
            if let Some(dense_set) =  st.as_any_mut().downcast_mut::<DenseSet<T>>() {
                for (id, item) in dense_set {
                    if self.entities.len() <= id as usize {
                        continue;
                    }
                    if let Some(entity) = self.entities[id as usize].clone() {
                        f(entity, item);
                    }
                }
            }
        }
    }
    
    pub fn query<T: Component + 'static>(&self, mut f: impl FnMut(Entity, &T) -> ()) {
        if let Some(st) = self.sets.get(&TypeId::of::<T>()) {
            let dense_set = st.as_any().downcast_ref::<DenseSet<T>>();
            if let Some(dense_set) = dense_set {
                // 获取实体这里不太好，后续看能否优化下
                for (id, item) in dense_set {
                    if self.entities.len() <= id as usize {
                        continue; 
                    }
                    if let Some(entity) = self.entities[id as usize].clone() {
                        f(entity, item);
                    }
                }
            }
        }
    }
    
    pub fn add_entity(&mut self, entity: Entity) {
        let entity_id = entity.id as usize;
        if entity_id < self.entities.len() {
            if let Some(entity) = self.entities[entity_id].clone() {
                if entity.generation.get() >= entity.generation.get() {
                    return;
                }
            }
            self.entities[entity_id] = Some(entity);
        } else {
            let new_size = entity_id + 1;
            self.entities.resize(new_size, None);
            self.entities[entity_id] = Some(entity);
        }
    }
    
    pub fn remove_entity(&mut self, entity: Entity) -> bool {
        if !self.is_valid_entity(&entity) {
            return false;
        }
        self.entities[entity.id as usize] = None;
        for set in self.sets.values_mut() {
            set.remove(entity.id);
        }
        true
    }
    
    pub fn add_component<T: Component + 'static>(&mut self, entity: &Entity, component: T) {
        if !self.is_valid_entity(&entity) {
            return; 
        }

        if let Some(st) = self.sets.get_mut(&TypeId::of::<T>()) {
            if let Some(dense_set) = st.as_any_mut().downcast_mut::<DenseSet<T>>() {
                dense_set.add(entity.id as u32, component);
            }
        }
    }
    
    fn is_valid_entity(&self, entity: &Entity) -> bool {
        if entity.id >= self.entities.len() as u32  {
            return false;
        }

        if let Some(e) = &self.entities[entity.id as usize] {
            if e.generation.get() != entity.generation.get() {
                return false;
            }
            return true;
        }
        false
    }
    
}