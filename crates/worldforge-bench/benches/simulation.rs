//! World Forge performance benchmarks.
//!
//! Measures throughput of the core simulation subsystems:
//! - Fixed64 arithmetic
//! - ECS operations (insert, query, fingerprint)
//! - Economy tick (production + transfer)
//! - Full simulation run (supply-chain example)
//! - Proof chain computation
//! - Package fingerprinting

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use worldforge_core::hash::{Fingerprint, FingerprintBuilder};
use worldforge_core::{EntityId, Fixed64, Tick};
use worldforge_ecs::SimulationWorld;
use worldforge_world::{Inventory, ProductionRule};

fn bench_fixed64_arithmetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixed64");

    let a = Fixed64::from_int(12345);
    let b = Fixed64::from_int(67890);

    group.bench_function("add", |bencher| {
        bencher.iter(|| black_box(a) + black_box(b))
    });

    group.bench_function("mul", |bencher| {
        bencher.iter(|| black_box(a) * black_box(b))
    });

    group.bench_function("div", |bencher| {
        bencher.iter(|| black_box(a) / black_box(b))
    });

    let frac = Fixed64::from_f64_lossy(std::f64::consts::PI);
    group.bench_function("mul_fractional", |bencher| {
        bencher.iter(|| black_box(frac) * black_box(frac))
    });

    group.finish();
}

fn bench_ecs_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("ecs");

    group.bench_function("spawn_100", |bencher| {
        bencher.iter(|| {
            let mut world = SimulationWorld::new();
            for i in 0..100u64 {
                let id = EntityId::deterministic(42, i);
                world.spawn(id);
            }
            black_box(&world);
        })
    });

    group.bench_function("insert_component_100", |bencher| {
        let mut world = SimulationWorld::new();
        let ids: Vec<_> = (0..100u64)
            .map(|i| {
                let id = EntityId::deterministic(42, i);
                world.spawn(id);
                id
            })
            .collect();
        bencher.iter(|| {
            for id in &ids {
                let mut inv = Inventory::new();
                inv.set("ore", Fixed64::from_int(100));
                world.insert_component(*id, inv);
            }
            black_box(&world);
        })
    });

    group.bench_function("get_component_100", |bencher| {
        let mut world = SimulationWorld::new();
        let ids: Vec<_> = (0..100u64)
            .map(|i| {
                let id = EntityId::deterministic(42, i);
                world.spawn(id);
                let mut inv = Inventory::new();
                inv.set("ore", Fixed64::from_int(100));
                world.insert_component(id, inv);
                id
            })
            .collect();
        bencher.iter(|| {
            for id in &ids {
                black_box(world.get_component::<Inventory>(id));
            }
        })
    });

    group.bench_function("fingerprint_100_entities", |bencher| {
        let mut world = SimulationWorld::new();
        for i in 0..100u64 {
            let id = EntityId::deterministic(42, i);
            world.spawn(id);
            let mut inv = Inventory::new();
            inv.set("ore", Fixed64::from_int(i as i32));
            world.insert_component(id, inv);
        }
        bencher.iter(|| black_box(world.fingerprint()))
    });

    group.finish();
}

fn bench_economy_tick(c: &mut Criterion) {
    let mut group = c.benchmark_group("economy");

    for entity_count in [10, 50, 100, 500] {
        group.bench_with_input(
            BenchmarkId::new("production", entity_count),
            &entity_count,
            |bencher, &n| {
                let mut world = SimulationWorld::new();
                let entities: Vec<_> = (0..n as u64)
                    .map(|i| {
                        let id = EntityId::deterministic(42, i);
                        world.spawn(id);
                        let mut inv = Inventory::new();
                        inv.set("ore", Fixed64::from_int(10000));
                        inv.set("energy", Fixed64::from_int(10000));
                        world.insert_component(id, inv);
                        world.insert_component(
                            id,
                            ProductionRule {
                                name: "smelt".to_string(),
                                inputs: vec![
                                    ("ore".to_string(), Fixed64::from_int(2)),
                                    ("energy".to_string(), Fixed64::from_int(1)),
                                ],
                                outputs: vec![("steel".to_string(), Fixed64::from_int(1))],
                                capacity: Fixed64::ONE,
                                energy_cost: Fixed64::ZERO,
                            },
                        );
                        (id, format!("entity_{i}"))
                    })
                    .collect();

                bencher.iter(|| {
                    let mut events = Vec::new();
                    worldforge_economy::run_production(
                        &mut world,
                        &entities,
                        Tick::new(1),
                        &mut events,
                    );
                    black_box(&events);
                })
            },
        );
    }

    group.finish();
}

fn bench_proof_chain(c: &mut Criterion) {
    let mut group = c.benchmark_group("proof");

    for event_count in [100, 1000, 10000] {
        group.bench_with_input(
            BenchmarkId::new("hash_chain", event_count),
            &event_count,
            |bencher, &n| {
                let events: Vec<Vec<u8>> = (0..n)
                    .map(|i| format!("event_{}", i).into_bytes())
                    .collect();
                bencher.iter(|| {
                    let mut builder = FingerprintBuilder::new();
                    for event in &events {
                        builder.update(event);
                    }
                    black_box(builder.finalize())
                })
            },
        );
    }

    group.bench_function("fingerprint_32b", |bencher| {
        let data = vec![0xABu8; 32];
        bencher.iter(|| black_box(Fingerprint::hash(&data)))
    });

    group.bench_function("fingerprint_1kb", |bencher| {
        let data = vec![0xABu8; 1024];
        bencher.iter(|| black_box(Fingerprint::hash(&data)))
    });

    group.bench_function("fingerprint_1mb", |bencher| {
        let data = vec![0xABu8; 1024 * 1024];
        bencher.iter(|| black_box(Fingerprint::hash(&data)))
    });

    group.finish();
}

fn bench_full_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("simulation");
    group.sample_size(10); // Full runs are expensive

    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let world_path = repository_root.join("examples/supply-chain");
    if world_path.exists() {
        for ticks in [100, 500, 1000] {
            group.bench_with_input(
                BenchmarkId::new("supply_chain", ticks),
                &ticks,
                |bencher, &t| {
                    bencher.iter(|| {
                        let mut runtime = worldforge_runtime::SimulationRuntime::load_bounded(
                            &world_path,
                            42,
                            Some(t),
                            0,
                        )
                        .unwrap();
                        let result = runtime.run().unwrap();
                        black_box(result);
                    })
                },
            );
        }
    }

    let stress_path = repository_root.join("examples/stress-test");
    if stress_path.exists() {
        for ticks in [100, 1_000] {
            group.bench_with_input(
                BenchmarkId::new("stress_40_entities", ticks),
                &ticks,
                |bencher, &t| {
                    bencher.iter(|| {
                        let mut runtime = worldforge_runtime::SimulationRuntime::load_bounded(
                            &stress_path,
                            42,
                            Some(t),
                            0,
                        )
                        .unwrap();
                        black_box(runtime.run().unwrap());
                    })
                },
            );
        }
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_fixed64_arithmetic,
    bench_ecs_operations,
    bench_economy_tick,
    bench_proof_chain,
    bench_full_simulation,
);
criterion_main!(benches);
