use crate::{exec, parity_wasm_utils, wasmi_utils};
use parity_wasm::elements::{Internal, Module};
use rand::Rng;

pub mod transform;

#[allow(dead_code)]
pub struct Optimizer {
    module: Module,
}

impl Optimizer {
    pub fn new(module: Module) -> Self {
        Optimizer { module }
    }

    pub fn run(&self) {}

    /// Finds a module that has functions equivalent to the functions in the given module.
    pub fn synthesize(&self, rng: &mut impl Rng) {
        // Module in wasmi, WASM interpreter.
        let wasmi_module = wasmi::Module::from_parity_wasm_module(self.module.clone())
            .expect("Failed to load parity-wasm Module.");
        let instance = wasmi::ModuleInstance::new(&wasmi_module, &wasmi::ImportsBuilder::default())
            .expect("Failed to instantiate wasm module.")
            .assert_no_start();

        let export_section = self
            .module
            .export_section()
            .expect("Module doesn't have export section.");
        let num_imports = parity_wasm_utils::import_entries_len(&self.module);

        for export_entry in export_section.entries() {
            if let Internal::Function(idx) = export_entry.internal() {
                let func_name = export_entry.field();
                let func_ref = wasmi_utils::func_by_name(&instance, func_name).unwrap();

                let _test_cases = exec::generate_test_cases(rng, &func_ref);

                let _signature = func_ref.signature();
                // The index of this function in function section, and code section.
                let _func_idx = *idx as usize - num_imports;
                // Get the type signature
                // generate random candidate function
            }
        }
        // for func in module.functions {
        //   let test_cases = generate_test_cases(func)
        //   let candidate = generate_random_func(func.type);
        //     while (!candidate.validate()) {
        //       do_transform(candidate)
        //       if !exec_test_cases(func, test_cases) {
        //          continue
        //       }
        //       match verify(candidate, func) {
        //          Verified => break,
        //          CounterExample(inputs) => {
        //              let exepected_output = func.invoke_with(inputs);
        //              test_cases.push((inputs, output))
        //          }
        //       }
        //    }
        // }
    }
}
