use criterion::{criterion_group, criterion_main, Criterion};

fn perft_benchmark(_c: &mut Criterion) {
    // Benchmarks will be added in Task 7 (movegen + perft).
}

criterion_group!(benches, perft_benchmark);
criterion_main!(benches);
