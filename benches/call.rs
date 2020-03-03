use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::prelude::*;

fn bench_call(c: &mut Criterion) {
    let mut rng = thread_rng();

    let raw_input = rng.gen::<i32>();
    let wasmer_input = vec![wasmer_runtime::Value::I32(raw_input)];

    let file = "p17";
    let binary: Vec<u8> =
        wat::parse_file(["./examples/hackers_delight/", file, ".wat"].concat()).unwrap();

    // WASMER
    let import_object = wasmer_runtime::imports! {};
    let instance = wasmer_runtime::instantiate(&binary, &import_object).unwrap();
    let func = instance.dyn_func(file).unwrap();
    c.bench_function("wasmer", |b| {
        b.iter(|| func.call(black_box(&wasmer_input)).unwrap())
    });

    let module = wasmi::Module::from_buffer(&binary).unwrap();
    let instance = wasmi::ModuleInstance::new(&module, &wasmi::ImportsBuilder::default())
        .unwrap()
        .assert_no_start();
    let wasmi_input = vec![wasmi::RuntimeValue::I32(raw_input)];
    c.bench_function("wasmi", |b| {
        b.iter(|| {
            instance
                .invoke_export(file, &wasmi_input, &mut wasmi::NopExternals)
                .unwrap()
        })
    });
}

criterion_group!(benches, bench_call);
criterion_main!(benches);
