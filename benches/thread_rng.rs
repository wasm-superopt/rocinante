use criterion::{criterion_group, criterion_main, Criterion};
use rand::Rng;

fn thread_rng_inner() {
    let _result = rand::thread_rng().gen::<i32>();
}

fn thread_rng_outer<R: Rng + ?Sized>(rng: &mut R) {
    let _result = rng.gen::<i32>();
}

fn bench_thread_rng(c: &mut Criterion) {
    let mut group = c.benchmark_group("ThreadRng");

    let mut rng = rand::thread_rng();
    let _ = rng.gen::<i32>();

    group.bench_function("thread_rng_outer", |b| {
        b.iter(|| thread_rng_outer(&mut rng))
    });
    group.bench_function("thread_rng_inner", |b| b.iter(|| thread_rng_inner()));
}

criterion_group!(benches, bench_thread_rng);
criterion_main!(benches);
