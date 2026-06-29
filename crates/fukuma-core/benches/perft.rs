use criterion::{criterion_group, criterion_main, Criterion};
use fukuma_core::{movegen::perft, position::Position};

fn bench_perft(c: &mut Criterion) {
    c.bench_function("perft startpos depth4", |b| {
        b.iter(|| {
            let mut pos = Position::startpos();
            perft(&mut pos, 4)
        })
    });
}

criterion_group!(benches, bench_perft);
criterion_main!(benches);
