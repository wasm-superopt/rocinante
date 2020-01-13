use parity_wasm::elements::{ExportSection, Internal, Module};

fn get_func_name(export_section: &ExportSection, func_idx: u32) -> Option<&str> {
    let export_entries = export_section.entries();

    for entry in export_entries {
        if let Internal::Function(idx) = entry.internal() {
            if *idx == func_idx {
                return Some(entry.field());
            }
        }
    }

    None
}

pub fn print_functions(module: Module) {
    let function_section = module
        .function_section()
        .expect("No function section in the module.");

    let code_section = module
        .code_section()
        .expect("No code section in the module.");

    let type_section = module
        .type_section()
        .expect("No type section in the module.");

    let export_section = module
        .export_section()
        .expect("No export section in the module.");

    // We assume that the number of function signatures and function bodies are
    // the same in the module and the ordering is also the same.
    assert!(function_section.entries().len() == code_section.bodies().len());

    let num_func = function_section.entries().len();
    for i in 0..num_func {
        let func_sig = function_section.entries()[i];
        let typ_idx = func_sig.type_ref();
        let typ = &type_section.types()[typ_idx as usize];

        let name_opt = get_func_name(export_section, i as u32);

        if let Some(func_name) = name_opt {
            println!("{}", func_name);
        } else {
            println!("Function name not found.");
        }

        println!("{:?}", typ);

        let func_body = &code_section.bodies()[i];
        println!("{:?}", func_body.locals());

        for instr in func_body.code().elements() {
            println!("{}", instr);
        }

        println!();
    }
}
