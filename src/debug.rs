use parity_wasm::elements::{Internal, Module, Type};

fn import_entries_len(module: &Module) -> usize {
    match module.import_section() {
        Some(import_section) => import_section.entries().len(),
        None => 0,
    }
}

fn get_func_name(module: &Module, func_idx: usize) -> Option<&str> {
    // https://webassembly.github.io/spec/core/syntax/modules.html#imports
    // In each index space, the indices of imports go before the first index of
    // any definition contained in the module itself.
    let import_entries_len = import_entries_len(module);
    let export_section = module
        .export_section()
        .expect("No export section in the module.");

    for entry in export_section.entries() {
        if let Internal::Function(idx) = entry.internal() {
            if *idx == (func_idx + import_entries_len) as u32 {
                return Some(entry.field());
            }
        }
    }

    None
}

fn get_func_type(module: &Module, typ_idx: usize) -> &Type {
    let type_section = module
        .type_section()
        .expect("No type section in the module.");

    &type_section.types()[typ_idx]
}

pub fn print_functions(module: &Module) {
    let function_section = module
        .function_section()
        .expect("No function section in the module.");

    let code_section = module
        .code_section()
        .expect("No code section in the module.");

    // We assume that the number of function signatures and function bodies are
    // the same in the module and the ordering is also the same.
    assert!(function_section.entries().len() == code_section.bodies().len());

    let num_func = function_section.entries().len();
    for i in 0..num_func {
        let func_sig = function_section.entries()[i];
        let typ_idx = func_sig.type_ref();
        let typ = get_func_type(module, typ_idx as usize);

        let name_opt = get_func_name(module, i);

        if let Some(func_name) = name_opt {
            println!("{}", func_name);
        } else {
            println!("(anonymous function)");
        }

        // Currently there is only one type.
        let Type::Function(func_type) = typ;
        println!(
            "param types: {:?}, return type: {:?}",
            func_type.params(),
            func_type.return_type()
        );

        let func_body = &code_section.bodies()[i];
        if !func_body.locals().is_empty() {
            println!("{:?}", func_body.locals());
        }

        for instr in func_body.code().elements() {
            println!("{}", instr);
        }

        println!();
    }
}
