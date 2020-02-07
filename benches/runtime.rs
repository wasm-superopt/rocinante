use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::prelude::*;

fn wasmtime_invoke(binary: &[u8], func_name: &str, inputs: &[i32]) {
    use wasmtime::*;

    let store = wasmtime::Store::default();
    let module = Module::new(&store, &binary).unwrap();
    let instance = Instance::new(&store, &module, &[]).unwrap();

    let func = instance
        .find_export_by_name(func_name)
        .expect(func_name)
        .func()
        .unwrap();

    for input in inputs {
        let _result = func.borrow().call(&[wasmtime::Val::I32(*input)]).unwrap();
    }
}

fn wasmi_invoke(binary: &[u8], func_name: &str, inputs: &[i32]) {
    use wasmi::*;

    let module = wasmi::Module::from_buffer(&binary).expect("failed to load wasm.");
    let instance = ModuleInstance::new(&module, &ImportsBuilder::default())
        .unwrap()
        .assert_no_start();
    for input in inputs {
        let _result = instance
            .invoke_export(func_name, &[RuntimeValue::I32(*input)], &mut NopExternals)
            .unwrap();
    }
}

fn wasmer_invoke(binary: &[u8], func_name: &str, inputs: &[i32]) {
    use wasmer_runtime::*;

    let import_object = imports! {};
    let instance = instantiate(binary, &import_object).unwrap();

    for input in inputs {
        let _result = instance
            .dyn_func(func_name)
            .unwrap()
            .call(&[Value::I32(*input)])
            .unwrap();
    }
}

fn bench_invoke(c: &mut Criterion) {
    let mut group = c.benchmark_group("Invoke");

    let binary: Vec<u8> = wat::parse_file("./examples/hackers_delight/p1.wat").unwrap();
    let mut rng = thread_rng();
    for size in [4, 8, 16, 32, 64].iter() {
        let size = *size as usize;
        let mut inputs = Vec::with_capacity(size);
        for _ in 0..size {
            inputs.push(rng.gen::<i32>());
        }

        group.bench_with_input(BenchmarkId::new("wasmtime", size), &inputs, |b, i| {
            b.iter(|| wasmtime_invoke(&binary, "p1", i))
        });
        group.bench_with_input(BenchmarkId::new("wasmi", size), &inputs, |b, i| {
            b.iter(|| wasmi_invoke(&binary, "p1", i))
        });
        group.bench_with_input(BenchmarkId::new("wasmer", size), &inputs, |b, i| {
            b.iter(|| wasmer_invoke(&binary, "p1", i))
        });
    }
}

criterion_group!(benches, bench_invoke);
criterion_main!(benches);
