//! 通过公开的 `World` 接口验证稀疏集支撑的增加、删除与实体回收行为。
//!
//! 这里不直接访问 `DenseSet` 的内部字段，而是把查询结果
//! “实体 -> 组件值” 与测试维护的模型逐项对比。
//! 这样既能发现 dense 中的数据错误，也能发现实体 ID 与组件配对错位。

use std::cell::RefCell;

use decs::entity::Entity;
use decs::world::World;

#[derive(Debug)]
struct Position(i32);

#[derive(Debug)]
struct Velocity(i32);

/// 保留重复项的收集方式：如果同一实体被存了两份组件，
/// 结果里会出现两条记录，而不是像 Map 那样互相覆盖。
fn collect_positions(world: &World) -> Vec<(String, i32)> {
    let out = RefCell::new(Vec::new());
    world.query::<Position>(|entity, position| {
        out.borrow_mut().push((entity.to_string(), position.0));
    });
    let mut result = out.into_inner();
    result.sort();
    result
}

fn collect_velocities(world: &World) -> Vec<(String, i32)> {
    let out = RefCell::new(Vec::new());
    world.query::<Velocity>(|entity, velocity| {
        out.borrow_mut().push((entity.to_string(), velocity.0));
    });
    let mut result = out.into_inner();
    result.sort();
    result
}

fn expected(entities: &[(Entity, i32)]) -> Vec<(String, i32)> {
    let mut result: Vec<(String, i32)> = entities
        .iter()
        .map(|(entity, value)| (entity.to_string(), *value))
        .collect();
    result.sort();
    result
}

fn has_entry(entries: &[(String, i32)], entity: &Entity) -> bool {
    entries.iter().any(|(owner, _)| owner == &entity.to_string())
}

fn value_of(entries: &[(String, i32)], entity: &Entity) -> Option<i32> {
    entries
        .iter()
        .find(|(owner, _)| owner == &entity.to_string())
        .map(|(_, value)| *value)
}

fn world_with_entities(values: &[i32]) -> (World, Vec<Entity>) {
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Velocity>();

    let mut entities = Vec::new();
    for &value in values {
        let entity = world.add_entity();
        world.add_component(&entity, Position(value));
        entities.push(entity);
    }
    (world, entities)
}

#[test]
fn add_query_returns_each_component_with_its_owner() {
    let values: Vec<i32> = (0..10).map(|i| i * 10).collect();
    let (world, entities) = world_with_entities(&values);

    let model: Vec<(Entity, i32)> = entities
        .iter()
        .cloned()
        .zip(values.iter().copied())
        .collect();

    assert_eq!(collect_positions(&world), expected(&model));
}

#[test]
fn adding_same_component_twice_keeps_only_latest_value() {
    let mut world = World::new();
    world.register::<Position>();

    let entity = world.add_entity();
    world.add_component(&entity, Position(1));
    world.add_component(&entity, Position(99));

    assert_eq!(
        collect_positions(&world),
        vec![(entity.to_string(), 99)]
    );
}

#[test]
fn removing_middle_entity_preserves_moved_last_pairing() {
    let (mut world, entities) = world_with_entities(&[0, 10, 20, 30, 40]);

    // 删除中间元素会触发 swap-remove：最后一个实体（值 40）会被搬到空位。
    world.remove_entity(entities[1].clone());
    let remaining = vec![
        (entities[0].clone(), 0),
        (entities[2].clone(), 20),
        (entities[3].clone(), 30),
        (entities[4].clone(), 40),
    ];
    assert_eq!(collect_positions(&world), expected(&remaining));

    // 再删除“被搬动过的”最后一个实体，验证 sparse 中记录的下标已同步修正。
    world.remove_entity(entities[4].clone());
    let remaining = vec![
        (entities[0].clone(), 0),
        (entities[2].clone(), 20),
        (entities[3].clone(), 30),
    ];
    assert_eq!(collect_positions(&world), expected(&remaining));

    // 依次删完剩余实体，队列应为空。
    world.remove_entity(entities[0].clone());
    world.remove_entity(entities[2].clone());
    world.remove_entity(entities[3].clone());
    assert_eq!(collect_positions(&world), Vec::new());
}

#[test]
fn removing_moved_entity_actually_removes_its_component() {
    let (mut world, entities) = world_with_entities(&[0, 10, 20, 30, 40]);

    world.remove_entity(entities[1].clone()); // 实体 4 的组件被搬到下标 1
    world.remove_entity(entities[4].clone()); // 删除这个被搬动过的组件

    // 把回收的两个 ID 都重新领出来：如果被搬动者的组件只是
    // “看起来删了”而实际还留在 dense 里，它会在新实体身上冒出来。
    let first_recycled = world.add_entity();
    let second_recycled = world.add_entity();
    assert!(!has_entry(&collect_positions(&world), &first_recycled));
    assert!(!has_entry(&collect_positions(&world), &second_recycled));
    assert_eq!(collect_positions(&world).len(), 3);
}

#[test]
fn removing_entity_removes_all_component_types() {
    let (mut world, entities) = world_with_entities(&[0, 10, 20, 30]);
    for (index, entity) in entities.iter().enumerate() {
        world.add_component(entity, Velocity(index as i32 * 100));
    }

    world.remove_entity(entities[1].clone());

    let remaining_entities: Vec<Entity> = vec![
        entities[0].clone(),
        entities[2].clone(),
        entities[3].clone(),
    ];
    let position_model: Vec<(Entity, i32)> = remaining_entities
        .iter()
        .cloned()
        .zip([0, 20, 30])
        .collect();
    let velocity_model: Vec<(Entity, i32)> = remaining_entities
        .iter()
        .cloned()
        .zip([0, 200, 300])
        .collect();

    assert_eq!(collect_positions(&world), expected(&position_model));
    assert_eq!(collect_velocities(&world), expected(&velocity_model));
}

#[test]
fn recycled_entity_does_not_inherit_old_components() {
    let (mut world, entities) = world_with_entities(&[10, 20]);
    let old = entities[0].clone();
    world.add_component(&old, Velocity(77));

    world.remove_entity(old);
    let recycled = world.add_entity();

    // 新实体复用 ID，但不应看到旧实体的 Position 或 Velocity。
    assert_eq!(collect_positions(&world).len(), 1);
    assert!(!has_entry(&collect_positions(&world), &recycled));
    assert!(!has_entry(&collect_velocities(&world), &recycled));

    world.add_component(&recycled, Position(999));
    assert_eq!(value_of(&collect_positions(&world), &recycled), Some(999));
}

#[test]
fn stale_handle_cannot_add_component_after_recycle() {
    let mut world = World::new();
    world.register::<Position>();

    let old = world.add_entity();
    let old_text = old.to_string();
    world.remove_entity(old.clone());

    let recycled = world.add_entity();
    assert_ne!(old_text, recycled.to_string(), "回收后 generation 应变化");

    // 旧句柄已经失效，不能把组件错误地加到复用了同一 ID 的新实体上。
    world.add_component(&old, Position(123));
    assert_eq!(collect_positions(&world), Vec::new());

    world.add_component(&recycled, Position(456));
    assert_eq!(
        collect_positions(&world),
        vec![(recycled.to_string(), 456)]
    );
}

#[test]
fn entity_without_components_is_recycled() {
    // 不注册任何组件、也不给实体加组件时，删除仍应成功并回收 ID。
    let mut world = World::new();

    let first = world.add_entity();
    let first_text = first.to_string();
    world.remove_entity(first);

    let second = world.add_entity();
    assert_ne!(first_text, second.to_string(), "应复用同一 ID 并递增 generation");
    assert!(
        second.to_string().contains("generation: 1"),
        "second entity should be generation 1, got {second}"
    );
}

#[test]
fn repeated_add_remove_churn_matches_model() {
    let (mut world, initial) = world_with_entities(
        &(0..128).map(|i| i * 3).collect::<Vec<_>>(),
    );

    // 第一轮：删除下标为 3 的倍数的实体。
    let mut live: Vec<(Entity, i32)> = Vec::new();
    for (index, entity) in initial.iter().enumerate() {
        if index % 3 == 0 {
            world.remove_entity(entity.clone());
        } else {
            live.push((entity.clone(), index as i32 * 3));
        }
    }
    assert_eq!(collect_positions(&world), expected(&live));

    // 第二轮：补进 64 个新实体（其中一部分应复用刚回收的 ID）。
    for i in 0..64 {
        let entity = world.add_entity();
        let value = 10_000 + i;
        world.add_component(&entity, Position(value));
        live.push((entity, value));
    }
    assert_eq!(collect_positions(&world), expected(&live));

    // 第三轮：删除前 40 个存活实体，再逐项核对剩余配对关系。
    let removed: Vec<Entity> = live.drain(..40).map(|(entity, _)| entity).collect();
    for entity in removed {
        world.remove_entity(entity);
    }
    assert_eq!(collect_positions(&world), expected(&live));
}
