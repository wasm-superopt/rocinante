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

// Cyclops: Intel Xeon Gold 6132 CPU @ 2.60GHz
//
// ThreadRng/thread_rng_outer
//                         time:   [3.5290 ns 3.5409 ns 3.5537 ns]
// Found 15 outliers among 100 measurements (15.00%)
//   15 (15.00%) low mild
// ThreadRng/thread_rng_inner
//                         time:   [4.2325 ns 4.2464 ns 4.2602 ns]
// Found 16 outliers among 100 measurements (16.00%)
//   16 (16.00%) low mild

// Personal Windows: Intel Core I7-8700K CPU @ 3.70GHz
// ThreadRng/thread_rng_outer
//                         time:   [2.8884 ns 2.8935 ns 2.8997 ns]
//                         change: [-1.6994% -1.0717% -0.4942%] (p = 0.00 < 0.05)
//                         Change within noise threshold.
// Found 7 outliers among 100 measurements (7.00%)
//   5 (5.00%) high mild
//   2 (2.00%) high severe
// ThreadRng/thread_rng_inner
//                         time:   [3.1590 ns 3.1669 ns 3.1761 ns]
//                         change: [-7.1228% -5.5819% -4.1729%] (p = 0.00 < 0.05)
//                         Performance has improved.
// Found 5 outliers among 100 measurements (5.00%)
//   3 (3.00%) high mild
//   2 (2.00%) high severe
