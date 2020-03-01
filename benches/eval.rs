use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::prelude::*;
use wasmer_runtime::*;

fn bench_call(c: &mut Criterion) {
    let mut rng = thread_rng();

    let input = vec![Value::I32(rng.gen::<i32>())];

    let file = "p17";
    let binary: Vec<u8> =
        wat::parse_file(["./examples/hackers_delight/", file, ".wat"].concat()).unwrap();

    let import_object = imports! {};
    let instance = instantiate(&binary, &import_object).unwrap();
    let func = instance.dyn_func(file).unwrap();

    c.bench_function("call", |b| b.iter(|| func.call(black_box(&input)).unwrap()));
}

criterion_group!(benches, bench_call);
criterion_main!(benches);
