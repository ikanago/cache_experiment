use common::Cache;
use criterion::{criterion_group, Criterion};
use sync_naive_lru::SyncNaiveLru;

fn insert() {
    let mut lru = SyncNaiveLru::new(4);
    lru.insert(1, 2);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("insert", |b| b.iter(|| insert()));
}

criterion_group!(sync_naive_lru_benches, criterion_benchmark);
