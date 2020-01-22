use crate::{exec, parity_wasm_utils, wasmi_utils};
use parity_wasm::elements::{
    FunctionType as EFunctionType, Instruction as EInstruction, Instructions as EInstructions,
    Internal as EInternal, Module as EModule, ValueType as EValueType,
};
use rand::Rng;

pub use self::transform::*;
mod transform;

#[allow(dead_code)]
pub struct Superoptimizer {
    module: EModule,
}

impl Superoptimizer {
    pub fn new(module: EModule) -> Self {
        Superoptimizer { module }
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
        let _num_imports = parity_wasm_utils::import_entries_len(&self.module);

        for export_entry in export_section.entries() {
            if let EInternal::Function(_idx) = export_entry.internal() {
                let func_name = export_entry.field();
                let func_ref = wasmi_utils::func_by_name(&instance, func_name).unwrap();

                let _test_cases = exec::generate_test_cases(rng, &func_ref);

                let _signature = func_ref.signature();

                //  loop {
                //      while(!generator.module.validate()) {
                //          generator.do_transform()
                //      }
                //      if eval_test_cases(generator.module, test_cases) > 0 {
                //          continue
                //      }
                //      match verify_equivalence(func_ref, generator.module.func()) {
                //          Verified => break,
                //          CounterExample(inputs) => {
                //              let expected_ouptut = func_ref.invoke_with(inputs);
                //              test_caes.push((inputs, output));
                //          }
                //      }
                //  }
            }
        }
    }
}

pub struct Generator {
    module: EModule,
}

impl Generator {
    pub fn new(func_type: &EFunctionType) -> Self {
        Self {
            module: gen_random_func(func_type),
        }
    }

    pub fn do_transform(&self) {
        let transform: Transform = rand::random();

        match transform {
            Transform::Opcode => {
                // Choose an instruction at random, and replace with a random,
                // equivalent one.
            }
            Transform::Operand => {
                // Select an instruction at random, and its operand is replaced by a
                // random operand drawn from an equivalence class of operands.
            }
            Transform::Swap => {
                // Select two instructions from the set of original instructions
                // union with Nop, and swap
            }
            Transform::Instruction => {
                // Select an instruction, and replace with a random instruction,
                // with random operands.
            }
        }
    }

    pub fn module(&self) -> &EModule {
        &self.module
    }
}

fn gen_random_func(func_type: &EFunctionType) -> EModule {
    let param_types = func_type.params();
    let return_type: Option<EValueType> = func_type.return_type();

    let instr: EInstruction = match return_type {
        None => EInstruction::End,
        Some(val_type) => match val_type {
            EValueType::I32 => EInstruction::I32Const(0),
            EValueType::I64 => EInstruction::I64Const(0),
            EValueType::F32 => EInstruction::F32Const(0),
            EValueType::F64 => EInstruction::F64Const(0),
        },
    };

    #[rustfmt::skip]
    let module = parity_wasm::builder::module()
        .export()
            .field("candidate")
            .internal()
            .func(0)
            .build()
        .function()
            .signature()
                .with_params(param_types.to_vec())
                .with_return_type(return_type)
                .build()
            .body()
                .with_instructions(EInstructions::new(vec![instr]))
                .build()
            .build()
        .build();

    module
}
