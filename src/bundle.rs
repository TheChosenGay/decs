use crate::entity::Entity;
use crate::storage::Component;
use crate::world::World;

pub trait Bundle {
    fn add_to(self, world: &mut World, entity: &Entity);
}

impl<T: Component + 'static> Bundle for T {
    fn add_to(self, world: &mut World, entity: &Entity) {
        world.add_component(entity, self);
    }
}

impl<T: Component + 'static> Bundle for (T,) {
    fn add_to(self, world: &mut World, entity: &Entity) {
        let (component,) = self;
        world.add_component(entity, component);
    }
}

macro_rules! impl_bundle_tuples {
    ($($t:ident),+) => {
        impl_bundle_tuples!(@go $($t),+);
    };

    (@go $head:ident, $next:ident) => {
        impl_bundle_tuples!(@emit $head, $next);
    };

    (@go $head:ident, $next:ident, $($rest:ident),+) => {
        impl_bundle_tuples!(@emit $head, $next, $($rest),+);
        impl_bundle_tuples!(@go $next, $($rest),+);
    };

    (@emit $head:ident, $($rest:ident),+) => {
        impl<$head: Component + 'static, $($rest: Component + 'static),+> Bundle for ($head, $($rest),+) {
            #[allow(non_snake_case)]
            fn add_to(self, world: &mut World, entity: &Entity) {
                let ($head, $($rest),+) = self;
                world.add_component::<$head>(entity, $head);
                $(
                    world.add_component(entity, $rest);
                )+
            }
        }
    };
}

impl_bundle_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
);
