extern crate parity_wasm;
extern crate rocinante;
extern crate wasmparser;

use parity_wasm::elements::{Module, Type};
use rocinante::verify;
use z3::SatResult;

fn read_wat(file: &str) -> Module {
    let binary = wat::parse_file(file).expect("Failed to read file");
    // Validate raw binary.
    wasmparser::validate(&binary, None /* Uses default parser config */)
        .expect("Failed to validate.");
    let module = Module::from_bytes(&binary).expect("Failed to deserialize.");
    module
}

fn get_func_type(module: &Module, typ_idx: usize) -> &Type {
    let type_section = module
        .type_section()
        .expect("No type section in the module.");

    &type_section.types()[typ_idx]
}

#[test]
fn verify_test() {
    let spec_module = read_wat("./examples/times-two/add.wat");
    let spec_function_section = spec_module.function_section().unwrap();
    let spec_code_section = spec_module.code_section().unwrap();

    assert_eq!(1, spec_function_section.entries().len());
    assert_eq!(1, spec_code_section.bodies().len());
    let spec_type = get_func_type(&spec_module, 0);
    let spec_body = &spec_code_section.bodies()[0];

    let candidate_module = read_wat("./examples/times-two/mul-two.wat");
    let candidate_function_section = candidate_module.function_section().unwrap();
    let candidate_code_section = candidate_module.code_section().unwrap();

    assert_eq!(1, candidate_function_section.entries().len());
    assert_eq!(1, candidate_code_section.bodies().len());
    let candidate_type = get_func_type(&candidate_module, 0);
    let candidate_body = &candidate_code_section.bodies()[0];

    assert_eq!(candidate_type, spec_type);
    let Type::Function(func_type) = spec_type;
    assert_eq!(
        SatResult::Sat,
        verify::verify(func_type, &spec_body, &candidate_body)
    );
}

#[test]
fn verify_fail_test() {
    let spec_module = read_wat("./examples/times-two/add.wat");
    let spec_function_section = spec_module.function_section().unwrap();
    let spec_code_section = spec_module.code_section().unwrap();

    assert_eq!(1, spec_function_section.entries().len());
    assert_eq!(1, spec_code_section.bodies().len());
    let spec_type = get_func_type(&spec_module, 0);
    let spec_body = &spec_code_section.bodies()[0];

    let candidate_module = read_wat("./examples/times-two/mul-three.wat");
    let candidate_function_section = candidate_module.function_section().unwrap();
    let candidate_code_section = candidate_module.code_section().unwrap();

    assert_eq!(1, candidate_function_section.entries().len());
    assert_eq!(1, candidate_code_section.bodies().len());
    let candidate_type = get_func_type(&candidate_module, 0);
    let candidate_body = &candidate_code_section.bodies()[0];

    assert_eq!(candidate_type, spec_type);
    let Type::Function(func_type) = spec_type;
    assert_eq!(
        SatResult::Unsat,
        verify::verify(func_type, &spec_body, &candidate_body)
    );
}
