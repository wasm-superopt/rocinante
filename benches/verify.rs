use criterion::{criterion_group, criterion_main, Criterion};
use rocinante::parity_wasm_utils;
use rocinante::solver::Z3Solver;

fn wat2module<S: AsRef<[u8]>>(source: S) -> parity_wasm::elements::Module {
    let binary = wabt::wat2wasm(source).expect("Failed to parse .wat");
    wasmparser::validate(&binary, None /* Uses default parser config */)
        .expect("Failed to validate.");
    parity_wasm::elements::Module::from_bytes(binary).expect("Failed to deserialize.")
}

fn bench_verify(c: &mut Criterion) {
    let spec_module: parity_wasm::elements::Module = wat2module(
        r#"(module
            (type $t0 (func (param i32) (result i32)))
            (func $add (type $t0) (param $p0 i32) (result i32)
              local.get $p0
              local.get $p0
              i32.add)
            (export "add" (func $add)))"#,
    );
    let (spec_func_type, spec_func_body) = parity_wasm_utils::func_by_name(&spec_module, "add");
    let candidate_module: parity_wasm::elements::Module = wat2module(
        r#"(module
            (type $t0 (func (param i32) (result i32)))
            (func $mul (type $t0) (param $p0 i32) (result i32)
              local.get $p0
              i32.const 2
              i32.mul)
            (export "mul" (func $mul)))"#,
    );
    let (candidate_func_type, candidate_func_body) =
        parity_wasm_utils::func_by_name(&candidate_module, "mul");
    assert_eq!(spec_func_type, candidate_func_type);

    let cfg = z3::Config::new();
    let ctx = z3::Context::new(&cfg);

    let solver = Z3Solver::new(&ctx, spec_func_type, spec_func_body);
    // [14.525 ms 14.865 ms 15.225 ms] on Taegyun's macbook pro
    c.bench_function("z3solver", |b| {
        b.iter(|| solver.verify(candidate_func_body.code().elements()))
    });
}

criterion_group!(benches, bench_verify);
criterion_main!(benches);
