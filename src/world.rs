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
        self.storage.add_component(entity, component);
    }
}

pub trait Query {
    type Item<'a>;
    fn for_each<F>(world: &World, f: F)
        where F: for<'a> FnMut(Entity, Self::Item<'a>);
}

// 为所有的component实现for each功能
impl <T: Component + 'static> Query for T {
    type Item<'a> = &'a T;
    fn for_each<F>(world: &World, f: F)
        where F: for<'a> FnMut(Entity, Self::Item<'a>)
    {
        world.storage.query::<T>(f);
    }
    
}

pub trait QueryMut {
    type Item<'a>; 
    fn for_each<F>(world: &mut World, f: F)
        where F: for<'a> FnMut(Entity, Self::Item<'a>);
}

impl <T: Component + 'static> QueryMut for T {
    type Item<'a> = &'a mut T;
    fn for_each<F>(world: &mut World, f: F)
        where F: for<'a> FnMut(Entity, Self::Item<'a>)
    {
        world.storage.query_mut::<T>(f);
    }
}

macro_rules! impl_query_tuples {
    ($($t:ident),+) => {
        impl_query_tuples!(@go $($t),+);
    };
    
    (@go $head:ident, $next:ident) => {
        impl_query_tuples!(@emit $head, $next);
    };
    
    (@go $head:ident, $next:ident, $($rest:ident),+) => {
        impl_query_tuples!(@emit $head, $next, $($rest),+);
        impl_query_tuples!(@go $next, $($rest),+);
    };

    (@emit $head:ident, $($tail:ident),+) => {
        // 为元组实现Query
        impl <$head: Component + 'static, $($tail: Component + 'static),+> Query for ($head, $($tail),+) {
            type Item<'a> = (&'a $head, $(&'a $tail),+);
            
            fn for_each<Visit>(world: &World, mut f: Visit)
                where Visit: for<'a> FnMut(Entity, Self::Item<'a>)
            {
                let Some(head_set) = world.storage.get::<$head>() else {
                    return;
                };
                $(
                    let Some($tail) = world.storage.get::<$tail>() else {
                        return;
                    };
                )+
                    
                for (id, h) in head_set {
                    if world.storage.get_entities().len() <= id as usize {
                        continue;
                    }
                    let Some(entity) = &world.storage.get_entities()[id as usize] else {
                       continue; 
                    };
                    
                    if let ($(Some($tail),)+) = ($($tail.get(id),)+) {
                        f(entity.clone(), (h, $($tail),+));
                    }
                }
            }
            
        }

        impl <$head: Component + 'static, $($tail: Component + 'static),+> QueryMut for ($head, $($tail),+) {
            type Item<'a> = (&'a mut $head, $(&'a mut $tail),+);

            #[allow(non_snake_case)]
            fn for_each<Visit>(world: &mut World, mut f: Visit)
                where Visit: for<'a> FnMut(Entity, Self::Item<'a>)
            {
                let Some(head_ptr) = world.storage.get_mut::<$head>().map(|set| set as *mut crate::dense_set::DenseSet<$head>) else {
                    return;
                };
                $(
                    let Some($tail) = world.storage.get_mut::<$tail>().map(|set| set as *mut crate::dense_set::DenseSet<$tail>) else {
                        return;
                    };
                )+

                // get_mut 的借用在这一句结束。指针留下，后面才能再取下一列。
                let head_set = unsafe { &mut *head_ptr };
                $(
                    let $tail = unsafe { &mut *$tail };
                )+

                for (id, h) in head_set {
                    if world.storage.get_entities().len() <= id as usize {
                        continue;
                    }
                    let Some(entity) = &world.storage.get_entities()[id as usize] else {
                        continue;
                    };
                    if let ($(Some($tail),)+) = ($($tail.get_mut(id),)+) {
                        f(entity.clone(), (h, $($tail),+));
                    }
                }
            }
        }

    };
}

impl_query_tuples!(A, B, C, D, E, F, G, H);

