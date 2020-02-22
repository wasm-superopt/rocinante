use parity_wasm::elements::{
    ExportEntry, ExportSection, Func, FuncBody, FunctionType, Internal, Module, Type,
};

pub fn export_by_name(export_section: &ExportSection, name: &str) -> Option<ExportEntry> {
    for entry in export_section.entries() {
        if entry.field() == name {
            return Some(entry.clone());
        }
    }
    None
}

pub fn import_entries_len(module: &Module) -> usize {
    match module.import_section() {
        Some(import_section) => import_section.entries().len(),
        None => 0,
    }
}

pub fn get_function_type<'module>(
    module: &'module Module,
    signature: &'module Func,
) -> &'module FunctionType {
    let type_idx = signature.type_ref() as usize;

    let type_section = module
        .type_section()
        .expect("Module doesn't contain type section.");
    let raw_type = &type_section.types()[type_idx];
    // There is only one type.
    let Type::Function(func_type) = raw_type;
    func_type
}

pub fn func_by_name<'module>(
    module: &'module Module,
    func_name: &str,
) -> (&'module FunctionType, &'module FuncBody) {
    let export_section = module
        .export_section()
        .expect("Module doesn't contain export section.");
    let export_entry = export_by_name(export_section, func_name)
        .unwrap_or_else(|| panic!("Module doesn't have export {}", func_name));
    let mut func_idx = match export_entry.internal() {
        Internal::Function(idx) => *idx as usize,
        unexpected => panic!(
            "Export {} is not a function, but {:?}",
            func_name, unexpected
        ),
    };
    func_idx -= import_entries_len(module);

    let function_section = module
        .function_section()
        .expect("Module doens't contain function section.");
    let code_section = module
        .code_section()
        .expect("Module doesn't contain code section.");

    let func = &function_section.entries()[func_idx];
    let func_type = get_function_type(module, func);
    let func_body = &code_section.bodies()[func_idx];

    (func_type, func_body)
}

pub fn build_module(func_name: &str, func_type: &FunctionType, func_body: FuncBody) -> Module {
    #[rustfmt::skip]
    let module = parity_wasm::builder::module()
        .export()
            .field(func_name)
            .internal()
            .func(0)
            .build()
        .function()
            .signature()
                .with_params(func_type.params().to_vec())
                .with_return_type(func_type.return_type())
                .build()
            .body()
                .with_func(func_body)
                .build()
            .build()
        .build();

    module
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_wasm::elements::{Instruction, Instructions, ValueType};

    fn instantiate(module: parity_wasm::elements::Module) -> wasmi::ModuleRef {
        let module =
            wasmi::Module::from_parity_wasm_module(module).expect("Failed to load wasmi module.");
        wasmi::ModuleInstance::new(&module, &wasmi::ImportsBuilder::default())
            .expect("Failed to build wasmi module instance.")
            .assert_no_start()
    }

    #[test]
    fn build_module_test() {
        let func_type = FunctionType::new(vec![ValueType::I32], Some(ValueType::I32));
        let func_body = FuncBody::new(
            vec![],
            Instructions::new(vec![
                Instruction::GetLocal(0),
                Instruction::GetLocal(0),
                Instruction::I32Add,
                Instruction::End,
            ]),
        );

        let add_module = build_module("add", &func_type, func_body);
        let instance = instantiate(add_module);
        assert_eq!(
            instance
                .invoke_export(
                    "add",
                    &[wasmi::RuntimeValue::I32(3)],
                    &mut wasmi::NopExternals,
                )
                .expect("failed to execute the function"),
            Some(wasmi::RuntimeValue::I32(6))
        );
    }

    #[test]
    fn build_module_empty() {
        let func_type = FunctionType::new(vec![ValueType::I32], Some(ValueType::I32));
        let func_body = FuncBody::new(vec![], Instructions::new(vec![]));

        let module = build_module("candidate", &func_type, func_body.clone());

        let expected_binary: Vec<u8> = vec![
            0, 97, 115, 109, 1, 0, 0, 0, 1, 6, 1, 96, 1, 127, 1, 127, 3, 2, 1, 0, 7, 13, 1, 9, 99,
            97, 110, 100, 105, 100, 97, 116, 101, 0, 0, 10, 3, 1, 1, 0,
        ];

        assert_eq!(module.to_bytes().unwrap(), expected_binary);
    }
}
