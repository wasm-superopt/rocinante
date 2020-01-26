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

#[allow(dead_code)]
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

    let func_sig = &function_section.entries()[func_idx];
    let func_type = get_function_type(module, func_sig);
    let func_body = &code_section.bodies()[func_idx];

    (func_type, func_body)
}
