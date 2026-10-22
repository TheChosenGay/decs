use decs::world::*;

struct Position {
    x: f32,
    y: f32
}

impl Position {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

fn main() {
    let mut world = World::new();
    

    world.register::<Position>();
    

    let player = world.add_entity();
    world.add_component(player, Position::new(0.0, 0.0));
    

    world.query::<Position>(|entity, position| {
        println!("Position of entity {}: ({}, {})", entity,  position.x, position.y);
    });
    
    world.query_mut::<Position>(|_, position| {
        position.x += 1.0;
        position.y += 1.0;
    });
    
    world.query::<Position>(|entity, position| {
        println!("Position of entity {}: ({}, {})", entity,  position.x, position.y);
    });
}

