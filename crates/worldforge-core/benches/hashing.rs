use criterion::{black_box, criterion_group, criterion_main, Criterion};
use worldforge_core::hash::{Fingerprint, FingerprintBuilder};

fn bench_fingerprint_hash(c: &mut Criterion) {
    let data = vec![0u8; 1024];
    c.bench_function("fingerprint_hash_1kb", |b| {
        b.iter(|| Fingerprint::hash(black_box(&data)))
    });

    let data = vec![0u8; 1024 * 1024];
    c.bench_function("fingerprint_hash_1mb", |b| {
        b.iter(|| Fingerprint::hash(black_box(&data)))
    });
}

fn bench_fingerprint_chain(c: &mut Criterion) {
    let a = Fingerprint::hash(b"chain test a");
    let b = Fingerprint::hash(b"chain test b");
    c.bench_function("fingerprint_chain", |b_iter| {
        b_iter.iter(|| black_box(a).chain(black_box(&b)))
    });
}

fn bench_fingerprint_builder(c: &mut Criterion) {
    let chunks: Vec<Vec<u8>> = (0..100).map(|i| vec![i as u8; 64]).collect();
    c.bench_function("fingerprint_builder_100_chunks", |b| {
        b.iter(|| {
            let mut builder = FingerprintBuilder::new();
            for chunk in &chunks {
                builder.update(chunk);
            }
            builder.finalize()
        })
    });
}

criterion_group!(
    benches,
    bench_fingerprint_hash,
    bench_fingerprint_chain,
    bench_fingerprint_builder,
);
criterion_main!(benches);
