//! 同条件对照：ECS query 与原生数组遍历的差距来自哪。
//!
//! 运行：`cargo run --release --example bench_query_fair`
//!
//! 对照组：
//! - 原生 Vec + 局部变量累加（最理想的直接遍历）
//! - 原生 Vec + Cell 累加（与 ECS 闭包相同的累加约束）
//! - ECS query + Cell 累加（现状）

use std::cell::Cell;
use std::hint::black_box;
use std::time::Instant;

use decs::world::World;

struct Position {
    x: f32,
    y: f32,
}

fn run_bench(label: &str, n: usize, iters: usize, mut f: impl FnMut() -> f64) {
    for _ in 0..3 {
        black_box(f());
    }
    let start = Instant::now();
    let mut checksum = 0.0f64;
    for _ in 0..iters {
        checksum += black_box(f());
    }
    let elapsed = start.elapsed();
    let total = (n * iters) as f64;
    let secs = elapsed.as_secs_f64();
    println!(
        "{label:<30} {:>8.2} ms  {:>6.2} ns/元素  {:>8.1} M元素/s  checksum={checksum:.1}",
        secs * 1e3,
        secs * 1e9 / total,
        total / secs / 1e6,
    );
}

fn main() {
    let n = 1_000_000usize;
    let iters = 20;

    let mut world = World::new();
    world.register::<Position>();
    for i in 0..n {
        let e = world.add_entity();
        world.add_component(&e, Position { x: i as f32, y: 1.0 });
    }

    let baseline: Vec<Position> = (0..n).map(|i| Position { x: i as f32, y: 1.0 }).collect();

    println!("=== n = {n}，遍历单个 Position 组件 ===");

    // 1) 原生 Vec，局部变量累加（编译器可放寄存器、可展开）
    run_bench("原生 Vec（局部累加）", n, iters, || {
        baseline.iter().map(|p| p.x as f64 + p.y as f64).sum()
    });

    // 2) 原生 Vec，Cell 累加（与 ECS 的 Fn 闭包相同约束）
    let sum_cell = Cell::new(0.0f64);
    run_bench("原生 Vec（Cell 累加）", n, iters, || {
        sum_cell.set(0.0);
        for p in baseline.iter() {
            sum_cell.set(sum_cell.get() + p.x as f64 + p.y as f64);
        }
        sum_cell.get()
    });

    // 3) ECS query，Cell 累加
    let sum_ecs = Cell::new(0.0f64);
    run_bench("ECS query（Cell 累加）", n, iters, || {
        sum_ecs.set(0.0);
        world.query::<Position>(|_, p| {
            sum_ecs.set(sum_ecs.get() + p.x as f64 + p.y as f64);
        });
        sum_ecs.get()
    });

    // 4) ECS query，局部变量累加（FnMut 后才可写，是与原生最公平的对照）
    run_bench("ECS query（局部累加）", n, iters, || {
        let mut sum = 0.0f64;
        world.query::<Position>(|_, p| {
            sum += p.x as f64 + p.y as f64;
        });
        sum
    });
}
