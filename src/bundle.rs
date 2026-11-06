use crate::entity::Entity;
use crate::storage::Component;
use crate::world::World;

pub trait Bundle {
    fn add_to(self, world: &mut World, entity: &Entity);
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
                world.get_storage_mut().add_component::<$head>(entity, $head);
                $(
                    world.get_storage_mut().add_component(entity, $rest);
                )+
            }
        }
    };
}

impl_bundle_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);