use crate::storage::Storage;
use crate::entity::Entity;
use crate::entity::EntitySpawner;
pub use crate::storage::Component;
pub use crate::query::{Query, QueryMut};

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
    
    #[allow(unused_mut)]
    pub fn query<Q: Query>(&self, mut f: impl for<'a>FnMut(Entity, Q::Item<'a>) -> ()) {
        Q::for_each(self,f);
    }
    
    #[allow(unused_mut)]
    pub fn query_mut<Q: QueryMut>(&mut self, mut f: impl for<'a>FnMut(Entity, Q::Item<'a>) -> ()) {
        Q::for_each(self,f);
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
        if !self.storage.has_component::<T>() {
            self.storage.register::<T>();
            return;
        }
        self.storage.add_component(entity, component);
    }
    
    pub(crate) fn get_storage(&self) -> &Storage {
        &self.storage
    }
    
    pub(crate) fn get_storage_mut(&mut self) -> &mut Storage {
        &mut self.storage
    }
}
