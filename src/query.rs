use crate::entity::Entity;
use crate::storage::Component;
use crate::world::World;

pub trait Query {
    type Item<'a>;
    fn for_each<F>(world: &World, f: F)
    where
        F: for<'a> FnMut(Entity, Self::Item<'a>);
}

// 为所有的component实现for each功能
impl<T: Component + 'static> Query for T {
    type Item<'a> = &'a T;
    fn for_each<F>(world: &World, f: F)
    where
        F: for<'a> FnMut(Entity, Self::Item<'a>),
    {
        world.get_storage().query::<T>(f);
    }
}

pub trait QueryMut {
    type Item<'a>;
    fn for_each<F>(world: &mut World, f: F)
    where
        F: for<'a> FnMut(Entity, Self::Item<'a>);
}

impl<T: Component + 'static> QueryMut for T {
    type Item<'a> = &'a mut T;
    fn for_each<F>(world: &mut World, f: F)
    where
        F: for<'a> FnMut(Entity, Self::Item<'a>),
    {
        world.get_storage_mut().query_mut::<T>(f);
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
                let Some(head_set) = world.get_storage().get::<$head>() else {
                    return;
                };
                $(
                    let Some($tail) = world.get_storage().get::<$tail>() else {
                        return;
                    };
                )+

                for (id, h) in head_set {
                    if world.get_storage().get_entities().len() <= id as usize {
                        continue;
                    }
                    let Some(entity) = &world.get_storage().get_entities()[id as usize] else {
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
                let Some(head_ptr) = world.get_storage_mut().get_mut::<$head>().map(|set| set as *mut crate::dense_set::DenseSet<$head>) else {
                    return;
                };
                $(
                    let Some($tail) = world.get_storage_mut().get_mut::<$tail>().map(|set| set as *mut crate::dense_set::DenseSet<$tail>) else {
                        return;
                    };
                )+

                // get_mut 的借用在这一句结束。指针留下，后面才能再取下一列。
                let head_set = unsafe { &mut *head_ptr };
                $(
                    let $tail = unsafe { &mut *$tail };
                )+

                for (id, h) in head_set {
                    if world.get_storage().get_entities().len() <= id as usize {
                        continue;
                    }
                    let Some(entity) = &world.get_storage().get_entities()[id as usize] else {
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
