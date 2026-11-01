use crate::storage::Storage;
use crate::entity::Entity;
use crate::entity::EntitySpawner;
pub use crate::storage::Component;

pub struct World {
    storage: Storage,
    entity_spawner: EntitySpawner,
}

impl World {
    pub fn new() -> Self {
        Self { storage: Storage::new(), entity_spawner: EntitySpawner::new() }
    }
    

    
    pub fn register<T: Component + 'static>(&mut self) {
        self.storage.register::<T>();
    }
    
    pub fn query<T: Component + 'static>(&self, mut f: impl FnMut(Entity, &T) -> ()) {
        self.storage.query::<T>(f);
    }
    
    pub fn query_mut<T: Component + 'static>(&mut self, f: impl FnMut(Entity, &mut T) -> ()) {
        self.storage.query_mut(f);
    }
    
    pub fn add_entity(&mut self) -> Entity {
        let entity = self.entity_spawner.spawn_entity();
        let ret_entity = entity.clone();
        self.storage.add_entity(entity);
        ret_entity
    }
    
    pub fn remove_entity(&mut self, entity: Entity) {
        if !self.storage.remove_entity(entity.clone()) {
            return;
        }
        // 回收entity
        self.entity_spawner.recycle_entity(entity);
    }
    
    pub fn add_component<T: Component + 'static>(&mut self, entity: &Entity, component: T) {
        self.storage.add_component(entity, component);
    }
}