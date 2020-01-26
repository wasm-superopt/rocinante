use crate::{debug, exec, parity_wasm_utils, solver, wasmi_utils};
use parity_wasm::elements::{
    FuncBody as EFuncBody, FunctionType as EFunctionType, Instruction as EInstruction,
    Instructions as EInstructions, Internal as EInternal, Module as EModule,
    ValueType as EValueType,
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
        // Module in wasmi, WASM interpreter. Instantiate this here and pass
        // down to exec module functions to avoid re-instantiation.
        let wasmi_module = wasmi::Module::from_parity_wasm_module(self.module.clone())
            .expect("Failed to load parity-wasm Module.");
        let instance = wasmi::ModuleInstance::new(&wasmi_module, &wasmi::ImportsBuilder::default())
            .expect("Failed to instantiate wasm module.")
            .assert_no_start();

        let export_section = self
            .module
            .export_section()
            .expect("Module doesn't have export section.");

        for export_entry in export_section.entries() {
            if let EInternal::Function(_idx) = export_entry.internal() {
                let func_name = export_entry.field();

                let test_cases = exec::generate_test_cases(rng, &instance, func_name);
                // let _generator = Generator::new(&func_type);
                let (func_type, func_body) =
                    parity_wasm_utils::func_by_name(&self.module, func_name);

                let cfg = z3::Config::new();
                let ctx = z3::Context::new(&cfg);
                let z3solver = solver::Z3Solver::new(&ctx, func_type, func_body);
                let mut generator = Generator::new(func_type);

                loop {
                    if generator.eval_test_cases(&test_cases) > 0 {
                        generator.do_transform(rng);
                        continue;
                    }
                    match z3solver.verify(generator.get_candidate_func()) {
                        solver::VerifyResult::Verified => {
                            // collect the function from generator
                            debug::print_functions(generator.module());
                            break;
                        }
                        solver::VerifyResult::CounterExample => {
                            // Add input, output pair to the test cases.
                            generator.do_transform(rng)
                        }
                    }
                }
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

    pub fn do_transform<R: Rng>(&mut self, rng: &mut R) {
        let transform: Transform = rng.gen::<Transform>();
        let instrs = self.get_candidate_func().code().elements();

        match transform {
            Transform::Opcode => {
                // Choose an instruction at random, and replace with a random,
                // equivalent one.

                let idx: usize = rng.gen_range(0, instrs.len());

                let new_instr = get_equiv(rng, &instrs[idx]);
                let mut new_instrs = Vec::with_capacity(instrs.len());
                new_instrs.clone_from_slice(instrs);
                new_instrs[idx] = new_instr;
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

    pub fn get_candidate_func(&self) -> &EFuncBody {
        let (_, candidate) = parity_wasm_utils::func_by_name(&self.module, "candidate");
        candidate
    }

    pub fn eval_test_cases(&self, test_cases: &[(exec::Input, exec::Output)]) -> u32 {
        // The module is validated this step.
        let module_or_err = wasmi::Module::from_parity_wasm_module(self.module.clone());
        if module_or_err.is_err() {
            // Compute the hamming distance
            return 10;
        }
        let module = module_or_err.unwrap();
        let instance_or_err =
            wasmi::ModuleInstance::new(&module, &wasmi::ImportsBuilder::default());
        if instance_or_err.is_err() {
            // Compute the hamming distance
            return 10;
        }
        let instance = instance_or_err.unwrap().assert_no_start();
        let candidate_func = wasmi_utils::func_by_name(&instance, "candidate").unwrap();

        let mut dist = 0;
        for (input, expected_output) in test_cases {
            let actual_output =
                wasmi::FuncInstance::invoke(&candidate_func, input, &mut wasmi::NopExternals);
            dist += exec::hamming_distance(expected_output, &actual_output);
        }

        dist
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
