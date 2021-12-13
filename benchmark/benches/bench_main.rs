use criterion::criterion_main;

mod sync_naive_lru;

criterion_main! {
    sync_naive_lru::sync_naive_lru_benches,
}
