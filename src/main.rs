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

struct Velocity {
    x: f32,
    y: f32
}

impl Velocity {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

fn main() {
    let mut world = World::new();
    

    world.register::<Position>();
    world.register::<Velocity>();
    

    let player = world.add_entity();
    world.add_component(&player, Position::new(0.0, 0.0));
    world.add_component(&player, Velocity::new(1.0, 2.0));
    

    world.query::<Position>(|entity, position| {
        println!("Position of entity {}: ({}, {})", entity,  position.x, position.y);
    });
    
    world.query_mut::<Position>(|_t, position| {
        position.x += 1.0;
        position.y += 1.0;
    });
    
    world.query::<Position>(|entity, position| {
        println!("Position of entity {}: ({}, {})", entity,  position.x, position.y);
    });
    
    world.query_mut::<Velocity>(|_, velocity| {
        println!("Velocity of entity before update ({}, {})", velocity.x, velocity.y);
        velocity.x += 1.0;
        velocity.y += 1.0;
    });
    
    world.query::<Velocity>(|entity, velocity| {
        println!("Velocity of entity {}: ({}, {})", entity,  velocity.x, velocity.y);
    });
}

