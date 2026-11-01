//! 稀疏集增加、删除与实体回收的性能基准（仅用 std）。
//!
//! 运行：`cargo run --release --example bench_sparse_ops`
//!
//! 场景：
//! - bulk_add：批量生成实体并添加 Position + Velocity
//! - bulk_remove_forward / reverse：按生成顺序 / 逆序整批删除
//! - churn_recycle：保持存活数不变，反复“删一个旧的、生成一个新（的并加组件）”
//!
//! 每轮都会打印最终 Position 数量，方便顺带观察删除后是否有残留。

use std::cell::Cell;
use std::collections::VecDeque;
use std::hint::black_box;
use std::time::Instant;

use decs::entity::Entity;
use decs::world::World;

struct Position {
    x: f32,
}

struct Velocity {
    x: f32,
}

fn build_world(n: usize) -> (World, Vec<Entity>) {
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Velocity>();
    let mut entities = Vec::with_capacity(n);
    for i in 0..n {
        let entity = world.add_entity();
        world.add_component(&entity, Position { x: i as f32 });
        world.add_component(&entity, Velocity { x: i as f32 });
        entities.push(entity);
    }
    (world, entities)
}

fn count_positions(world: &World) -> usize {
    let count = Cell::new(0usize);
    world.query::<Position>(|_, _| count.set(count.get() + 1));
    count.get()
}

fn count_velocities(world: &World) -> usize {
    let count = Cell::new(0usize);
    world.query::<Velocity>(|_, _| count.set(count.get() + 1));
    count.get()
}

fn report(label: &str, ops_per_rep: usize, reps: usize, total: std::time::Duration, extra: &str) {
    let total_ops = (ops_per_rep * reps) as f64;
    let secs = total.as_secs_f64();
    println!(
        "{label:<26} ops/rep={ops_per_rep:>8} reps={reps}  {:>9.2} ms  {:>8.2} ns/op  {:>8.2} Mop/s  {extra}",
        secs * 1e3,
        secs * 1e9 / total_ops,
        total_ops / secs / 1e6,
    );
}

fn bench_add(n: usize, reps: usize) {
    // 预热
    black_box(build_world(1_000).0);

    let start = Instant::now();
    for _ in 0..reps {
        let (world, entities) = build_world(n);
        black_box(entities.len());
        black_box(count_positions(&world));
    }
    report("bulk_add(实体+2组件)", n, reps, start.elapsed(), "");
}

fn bench_remove(n: usize, reps: usize, reverse: bool) {
    let label = if reverse {
        "bulk_remove_reverse"
    } else {
        "bulk_remove_forward"
    };
    let mut remaining_positions = 0usize;
    let mut remaining_velocities = 0usize;

    let start = Instant::now();
    for _ in 0..reps {
        let (mut world, mut entities) = build_world(n);
        if reverse {
            entities.reverse();
        }
        for entity in entities {
            world.remove_entity(entity);
        }
        remaining_positions = count_positions(&world);
        remaining_velocities = count_velocities(&world);
    }
    report(
        label,
        n,
        reps,
        start.elapsed(),
        &format!("删除后剩余 Position={remaining_positions} Velocity={remaining_velocities}"),
    );
}

fn bench_churn(n: usize, churn_ops: usize, reps: usize) {
    let mut final_positions = 0usize;
    let mut final_velocities = 0usize;

    let start = Instant::now();
    for _ in 0..reps {
        let (mut world, entities) = build_world(n);
        let mut live: VecDeque<Entity> = entities.into_iter().collect();
        for i in 0..churn_ops {
            let old = live.pop_front().expect("live queue should not be empty");
            world.remove_entity(old);
            let entity = world.add_entity();
            world.add_component(&entity, Position { x: i as f32 });
            world.add_component(&entity, Velocity { x: i as f32 });
            live.push_back(entity);
        }
        final_positions = count_positions(&world);
        final_velocities = count_velocities(&world);
        black_box(live.len());
    }
    report(
        "churn_recycle(删1加1)",
        churn_ops,
        reps,
        start.elapsed(),
        &format!("存活={n} 最终 Position={final_positions} Velocity={final_velocities}"),
    );
}

fn main() {
    let n = 100_000;
    println!("=== 稀疏集增删回收基准（n = {n}）===");
    bench_add(n, 5);
    bench_remove(n, 5, false);
    bench_remove(n, 5, true);
    bench_churn(n, 100_000, 3);
}
