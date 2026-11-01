//! 遍历组件访问的性能基准（仅用 std，不引入 criterion）。
//!
//! 运行：`cargo run --release --example bench_traversal`
//!
//! 对照设计：
//! - ECS 只读/可变遍历 vs 原生 `Vec<Position>` 遍历，后者是同一数据量的理想上限，
//!   两者差值即 Storage 封装（TypeId 查表、downcast、按 id 查 entities 并 clone Entity）的开销。
//! - 所有结果都汇入 checksum 并打印，防止编译器把遍历优化掉。

use std::cell::Cell;
use std::hint::black_box;
use std::time::Instant;

use decs::world::World;
use decs::Component;
use decs_macro::Component;

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
}

fn build_world(n: usize) -> World {
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Velocity>();
    for i in 0..n {
        let e = world.add_entity();
        world.add_component(&e, Position { x: i as f32, y: 1.0 });
        world.add_component(&e, Velocity { x: 1.0, y: 2.0 });
    }
    world
}

/// 跑 `iters` 轮 `f`，打印 ns/元素 与 M元素/s。`f` 每轮返回该轮的校验和。
fn run_bench(label: &str, n: usize, iters: usize, mut f: impl FnMut() -> f64) {
    // 预热：让分支预测、缓存、指令缓存进入稳态
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
        "{label:<28} n={n:>8} iters={iters:>4}  {:>9.2} ms  {:>7.2} ns/元素  {:>8.1} M元素/s  checksum={checksum:.1}",
        elapsed.as_secs_f64() * 1e3,
        secs * 1e9 / total,
        total / secs / 1e6,
    );
}

fn main() {
    // 每组规模总遍历量约 2000 万元素，使不同规模的计时可比且稳定
    for &n in &[10_000usize, 100_000, 1_000_000] {
        let iters = (20_000_000 / n).max(5);
        println!("\n=== 实体数 n = {n}（每个实体含 Position + Velocity）===");

        let mut world = build_world(n);
        let mut baseline: Vec<Position> =
            (0..n).map(|i| Position { x: i as f32, y: 1.0 }).collect();

        // 1) ECS 只读遍历 Position
        let sum = Cell::new(0.0f64);
        run_bench("ECS query<Position> 只读", n, iters, || {
            sum.set(0.0);
            world.query::<Position>(|_, p| sum.set(sum.get() + p.x as f64 + p.y as f64));
            sum.get()
        });

        // 2) ECS 可变遍历 Position（原地更新）
        let sum_mut = Cell::new(0.0f64);
        run_bench("ECS query_mut<Position> 更新", n, iters, || {
            sum_mut.set(0.0);
            world.query_mut::<Position>(|_, p| {
                p.x += 1.0;
                sum_mut.set(sum_mut.get() + p.x as f64);
            });
            sum_mut.get()
        });

        // 3) 对照：原生 Vec<Position> 只读遍历
        run_bench("原生 Vec<Position> 只读", n, iters, || {
            baseline.iter().map(|p| p.x as f64 + p.y as f64).sum()
        });

        // 4) 对照：原生 Vec<Position> 可变遍历
        run_bench("原生 Vec<Position> 更新", n, iters, || {
            let mut s = 0.0f64;
            for p in baseline.iter_mut() {
                p.x += 1.0;
                s += p.x as f64;
            }
            s
        });
    }
}
